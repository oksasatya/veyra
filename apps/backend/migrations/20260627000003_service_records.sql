CREATE TABLE service_records (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id   UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    service_date DATE NOT NULL,
    odometer     INTEGER NOT NULL,
    description  TEXT NOT NULL,
    workshop     TEXT,
    cost         NUMERIC(12,2),
    notes        TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_service_records_vehicle_id ON service_records(vehicle_id);
