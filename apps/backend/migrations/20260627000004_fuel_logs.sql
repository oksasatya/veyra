CREATE TABLE fuel_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id      UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    log_date        DATE NOT NULL,
    odometer        INTEGER NOT NULL,
    liters          NUMERIC(8,2) NOT NULL,
    price_per_liter NUMERIC(8,2) NOT NULL,
    total_cost      NUMERIC(12,2) GENERATED ALWAYS AS (liters * price_per_liter) STORED,
    station         TEXT,
    is_full_tank    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fuel_logs_vehicle_id ON fuel_logs(vehicle_id);
