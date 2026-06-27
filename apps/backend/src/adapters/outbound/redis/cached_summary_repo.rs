use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ports::repositories::{RepositoryResult, SummaryRepository, VehicleSummaryData};

use super::cache::RedisCache;

/// TTL for cached summary data — 60 seconds.
///
/// Staleness ≤ 60 s is accepted per spec (the CTE aggregation query is expensive;
/// TTL prevents hammering it on every request within a burst).
const SUMMARY_CACHE_TTL_SECS: u64 = 60;

/// Redis key prefix for summary cache entries.
const SUMMARY_KEY_PREFIX: &str = "cache:summary:";

/// Serde-serializable mirror of [`VehicleSummaryData`].
///
/// Lives entirely in this adapter file so that `VehicleSummaryData` (in `ports/`)
/// remains free of `serde` derives, honouring the hexagonal boundary rule
/// (`ports/` must not import `serde`).
///
/// `Decimal` is stored as its string representation — the canonical, lossless form
/// used by the HTTP layer as well, so no precision is lost on cache round-trip.
///
/// Reconstruction from cache is always fallible; on failure `get_vehicle_summary`
/// falls through to the inner repository (never panics).
///
/// # Big O (Bahasa Indonesia, four-beat)
///
/// **Apa yang terjadi:** setiap panggilan `get_vehicle_summary` pertama-tama cek
/// satu key di Redis (`cache:summary:{user_id}:{vehicle_id}`); kalau hit, langsung
/// return tanpa menyentuh Postgres.
///
/// **Kenapa bisa begitu:** key-nya deterministik dan unik per `(user_id, vehicle_id)` —
/// satu GET di Redis cukup. Postgres hanya dipanggil kalau key tidak ada atau sudah
/// kedaluwarsa.
///
/// **Konsekuensinya:** cache hit = O(1) Redis GET; miss = O(1) GET + CTE agregasi
/// di Postgres (yang sendiri O(n) terhadap jumlah record per vehicle, tapi one-shot
/// satu round-trip) + O(1) SET. TTL 60 s membatasi seberapa sering query mahal itu
/// dijalankan — satu burst request dalam 60 detik hanya menyentuh Postgres sekali.
///
/// **Solusinya:** pola TTL-only ini (tanpa version counter) cocok untuk data agregasi
/// yang jarang diinvalidasi secara presisi: staleness ≤ 60 s diterima, tidak ada scan
/// key, tidak ada INCR. Satu `SET EX 60` sudah cukup.
#[derive(Debug, Serialize, Deserialize)]
struct SummaryCacheModel {
    vehicle_id: Uuid,
    current_odometer: i32,
    total_services: i64,
    total_service_cost: String,
    total_refuels: i64,
    total_fuel_cost: String,
    total_expenses: String,
    upcoming_reminders: i64,
}

impl From<VehicleSummaryData> for SummaryCacheModel {
    fn from(d: VehicleSummaryData) -> Self {
        Self {
            vehicle_id: d.vehicle_id,
            current_odometer: d.current_odometer,
            total_services: d.total_services,
            total_service_cost: d.total_service_cost.to_string(),
            total_refuels: d.total_refuels,
            total_fuel_cost: d.total_fuel_cost.to_string(),
            total_expenses: d.total_expenses.to_string(),
            upcoming_reminders: d.upcoming_reminders,
        }
    }
}

impl SummaryCacheModel {
    /// Reconstruct a [`VehicleSummaryData`] from this cache model.
    ///
    /// Returns `None` if any `Decimal` string fails to parse — the cached data
    /// may be stale or corrupted. The caller falls through to the inner repo.
    /// This never panics on the production path.
    fn into_summary(self) -> Option<VehicleSummaryData> {
        let total_service_cost = self.total_service_cost.parse::<Decimal>().ok()?;
        let total_fuel_cost = self.total_fuel_cost.parse::<Decimal>().ok()?;
        let total_expenses = self.total_expenses.parse::<Decimal>().ok()?;
        Some(VehicleSummaryData {
            vehicle_id: self.vehicle_id,
            current_odometer: self.current_odometer,
            total_services: self.total_services,
            total_service_cost,
            total_refuels: self.total_refuels,
            total_fuel_cost,
            total_expenses,
            upcoming_reminders: self.upcoming_reminders,
        })
    }
}

// ── Decorator ─────────────────────────────────────────────────────────────────

/// Transparent caching decorator for [`SummaryRepository`].
///
/// Wraps any `SummaryRepository` and adds a Redis read-through cache with a
/// fixed 60-second TTL. No version counter, no explicit invalidation — the TTL
/// alone provides the recency guarantee (staleness ≤ 60 s is accepted per spec).
///
/// Cache key: `cache:summary:{user_id}:{vehicle_id}`
///
/// All cache operations are **fail-open**: any Redis error causes the request to
/// fall through to `inner` — correctness is never compromised, only latency.
/// No panic is possible on the production path.
pub struct CachedSummaryRepo {
    inner: Arc<dyn SummaryRepository>,
    cache: RedisCache,
}

impl CachedSummaryRepo {
    pub fn new(inner: Arc<dyn SummaryRepository>, cache: RedisCache) -> Self {
        Self { inner, cache }
    }

    /// Build the summary cache key, scoped per user so no cross-user bleed is
    /// possible. User A's summary for vehicle X is stored under a key that
    /// includes `user_id_A`; User B requesting the same `vehicle_id` would need
    /// to pass ownership auth (handled in the use case / handler layer), and
    /// even if they forged the request, they would generate a *different* cache
    /// key (`cache:summary:{user_id_B}:{vehicle_id}`), which will be a miss and
    /// go to the DB — which returns `NotFound`.
    fn summary_key(user_id: Uuid, vehicle_id: Uuid) -> String {
        format!("{SUMMARY_KEY_PREFIX}{user_id}:{vehicle_id}")
    }
}

#[async_trait]
impl SummaryRepository for CachedSummaryRepo {
    async fn get_vehicle_summary(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<VehicleSummaryData> {
        let key = Self::summary_key(user_id, vehicle_id);

        // Try cache hit — fail-open: any error or miss falls through to inner.
        if let Some(model) = self.cache.get_json::<SummaryCacheModel>(&key).await {
            if let Some(summary) = model.into_summary() {
                return Ok(summary);
            }
            // Reconstruction failed (stale / corrupted) — fall through to inner.
            tracing::debug!(
                key,
                "summary cache reconstruction failed — falling through to inner"
            );
        }

        // Cache miss (or reconstruction failure) → inner repo (Postgres).
        let summary = self.inner.get_vehicle_summary(vehicle_id, user_id).await?;

        // Populate cache (best-effort; never fails the caller).
        let model = SummaryCacheModel::from(VehicleSummaryData {
            vehicle_id: summary.vehicle_id,
            current_odometer: summary.current_odometer,
            total_services: summary.total_services,
            total_service_cost: summary.total_service_cost,
            total_refuels: summary.total_refuels,
            total_fuel_cost: summary.total_fuel_cost,
            total_expenses: summary.total_expenses,
            upcoming_reminders: summary.upcoming_reminders,
        });
        self.cache
            .set_json(&key, &model, SUMMARY_CACHE_TTL_SECS)
            .await;

        Ok(summary)
    }
}
