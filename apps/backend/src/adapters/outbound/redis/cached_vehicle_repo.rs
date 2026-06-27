use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::vehicle::{
        entity::Vehicle,
        value_objects::{FuelType, Odometer, PlateNumber},
    },
    ports::repositories::{
        CreateVehicleParams, RepositoryResult, UpdateVehicleParams, VehicleRepository,
    },
};

use super::cache::{RedisCache, VEHICLE_CACHE_TTL_SECS};

// ── Serde mirror ──────────────────────────────────────────────────────────────

/// Serde-serializable mirror of the `Vehicle` domain entity.
///
/// Value objects are stored as their primitive representations:
/// - `PlateNumber` → `String` (already normalized)
/// - `FuelType` → `&str` ("petrol" | "diesel" | "electric" | "hybrid")
/// - `Odometer` → `u32`
///
/// This mirror lives entirely in the adapter layer so that the domain entity
/// remains free of `serde` derives. Reconstruction from cache is fallible;
/// on failure the caller falls through to the inner repository.
#[derive(Debug, Serialize, Deserialize)]
struct VehicleCacheModel {
    id: Uuid,
    user_id: Uuid,
    brand: String,
    model: String,
    year: i16,
    plate_number: String,
    color: Option<String>,
    fuel_type: String,
    current_odometer: u32,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Vehicle> for VehicleCacheModel {
    fn from(v: Vehicle) -> Self {
        Self {
            id: v.id,
            user_id: v.user_id,
            brand: v.brand,
            model: v.model,
            year: v.year,
            plate_number: v.plate_number.as_str().to_owned(),
            color: v.color,
            fuel_type: v.fuel_type.as_str().to_owned(),
            current_odometer: v.current_odometer.value(),
            notes: v.notes,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}

impl VehicleCacheModel {
    /// Reconstruct a domain `Vehicle` from this cache model.
    ///
    /// Returns `None` if any value object fails validation — the data may be
    /// stale or corrupted; the caller should fall through to the inner repo.
    /// This never panics on the production path.
    fn into_vehicle(self) -> Option<Vehicle> {
        let plate_number = PlateNumber::new(self.plate_number).ok()?;
        let fuel_type = FuelType::parse(&self.fuel_type).ok()?;
        Some(Vehicle {
            id: self.id,
            user_id: self.user_id,
            brand: self.brand,
            model: self.model,
            year: self.year,
            plate_number,
            color: self.color,
            fuel_type,
            current_odometer: Odometer::new(self.current_odometer),
            notes: self.notes,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

// ── Decorator ─────────────────────────────────────────────────────────────────

/// Transparent caching decorator for [`VehicleRepository`].
///
/// Wraps any `VehicleRepository` and adds a Redis read-through cache keyed by
/// a per-user version counter. On every mutating operation (insert/update/delete)
/// the version is atomically incremented via `INCR`, which makes all of the
/// user's existing versioned list and detail cache keys unreachable without
/// requiring a key scan.
///
/// All cache operations are **fail-open**: a Redis error means the request
/// falls through to `inner` — correctness is never compromised, only latency.
///
/// # Big O (Bahasa Indonesia, four-beat)
///
/// **Apa yang terjadi:** operasi baca dimulai dengan satu GET ke Redis menggunakan key
/// yang sudah mengandung versi user; kalau hit, langsung return tanpa menyentuh
/// Postgres.
///
/// **Kenapa bisa begitu:** key-nya berformat `cache:v{ver}:vehicles:{uid}` — versi
/// di-prefix ke key, bukan di-lookup terpisah. Postgres hanya dipanggil kalau key
/// tidak ada (cache miss).
///
/// **Konsekuensinya:** read = O(1) Redis GET; miss → O(log n) Postgres index scan;
/// write = O(1) INCR yang langsung invalidasi semua read-key user tanpa scan satu
/// pun key di Redis.
///
/// **Solusinya:** pola ini (version-prefix invalidation) adalah cara paling efisien
/// untuk group-invalidation: satu INCR sudah cukup menggantikan ribuan DEL.
pub struct CachedVehicleRepo {
    inner: Arc<dyn VehicleRepository>,
    cache: RedisCache,
}

impl CachedVehicleRepo {
    pub fn new(inner: Arc<dyn VehicleRepository>, cache: RedisCache) -> Self {
        Self { inner, cache }
    }
}

/// Best-effort cleanup when `bump_version` fails — deletes the current-version
/// list and (optionally) a specific detail key so stale data isn't served.
///
/// This runs only on INCR failure; a failed DEL is logged at DEBUG and ignored.
/// The write operation itself has already succeeded; cache degradation must not
/// surface as an error to the caller.
async fn try_delete_stale_keys(cache: &RedisCache, user_id: Uuid, vehicle_id: Option<Uuid>) {
    if let Some(ver) = cache.version(user_id).await {
        let list_key = RedisCache::list_key(ver, user_id);
        cache.delete(&list_key).await;
        if let Some(vid) = vehicle_id {
            let detail_key = RedisCache::detail_key(ver, user_id, vid);
            cache.delete(&detail_key).await;
        }
    }
}

#[async_trait]
impl VehicleRepository for CachedVehicleRepo {
    async fn list_by_user(&self, user_id: Uuid) -> RepositoryResult<Vec<Vehicle>> {
        // Fix 1: None = Redis error → bypass cache entirely to avoid serving
        // a stale `cache:v0:…` entry from a degraded Redis instance.
        let ver = match self.cache.version(user_id).await {
            Some(v) => v,
            None => return self.inner.list_by_user(user_id).await,
        };
        let key = RedisCache::list_key(ver, user_id);

        // Try cache hit.
        if let Some(models) = self.cache.get_json::<Vec<VehicleCacheModel>>(&key).await {
            // Reconstruct domain entities; fall through to inner on any failure.
            let vehicles: Option<Vec<Vehicle>> = models
                .into_iter()
                .map(VehicleCacheModel::into_vehicle)
                .collect();
            if let Some(vehicles) = vehicles {
                return Ok(vehicles);
            }
        }

        // Cache miss (or reconstruction failed) → inner repo.
        let vehicles = self.inner.list_by_user(user_id).await?;

        // Populate the cache (best-effort; never fails the caller).
        let models: Vec<VehicleCacheModel> = vehicles
            .iter()
            .cloned()
            .map(VehicleCacheModel::from)
            .collect();
        self.cache
            .set_json(&key, &models, VEHICLE_CACHE_TTL_SECS)
            .await;

        Ok(vehicles)
    }

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<Vehicle> {
        // Fix 1: None = Redis error → bypass cache entirely.
        let ver = match self.cache.version(user_id).await {
            Some(v) => v,
            None => return self.inner.find_by_id(id, user_id).await,
        };
        let key = RedisCache::detail_key(ver, user_id, id);

        // Try cache hit.
        if let Some(model) = self.cache.get_json::<VehicleCacheModel>(&key).await {
            if let Some(vehicle) = model.into_vehicle() {
                return Ok(vehicle);
            }
        }

        // Cache miss → inner repo.
        let vehicle = self.inner.find_by_id(id, user_id).await?;

        // Populate the cache.
        self.cache
            .set_json(
                &key,
                &VehicleCacheModel::from(vehicle.clone()),
                VEHICLE_CACHE_TTL_SECS,
            )
            .await;

        Ok(vehicle)
    }

    async fn insert(&self, params: CreateVehicleParams) -> RepositoryResult<Vehicle> {
        let user_id = params.user_id;
        let vehicle = self.inner.insert(params).await?;
        // Fix 3: bump_version returns false on INCR error → best-effort stale
        // key cleanup so the write doesn't leave old data cached indefinitely.
        if !self.cache.bump_version(user_id).await {
            tracing::warn!(
                user_id = %user_id,
                "cache invalidation degraded after insert — attempting best-effort key deletion"
            );
            try_delete_stale_keys(&self.cache, user_id, None).await;
        }
        Ok(vehicle)
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        params: UpdateVehicleParams,
    ) -> RepositoryResult<Vehicle> {
        let vehicle = self.inner.update(id, user_id, params).await?;
        // Fix 3: best-effort on INCR failure.
        if !self.cache.bump_version(user_id).await {
            tracing::warn!(
                user_id = %user_id,
                vehicle_id = %id,
                "cache invalidation degraded after update — attempting best-effort key deletion"
            );
            try_delete_stale_keys(&self.cache, user_id, Some(id)).await;
        }
        Ok(vehicle)
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<()> {
        self.inner.delete(id, user_id).await?;
        // Fix 3: best-effort on INCR failure.
        if !self.cache.bump_version(user_id).await {
            tracing::warn!(
                user_id = %user_id,
                vehicle_id = %id,
                "cache invalidation degraded after delete — attempting best-effort key deletion"
            );
            try_delete_stale_keys(&self.cache, user_id, Some(id)).await;
        }
        Ok(())
    }
}
