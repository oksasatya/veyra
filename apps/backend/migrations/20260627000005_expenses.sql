CREATE TABLE expenses (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id   UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    expense_date DATE NOT NULL,
    category     TEXT NOT NULL,
    description  TEXT NOT NULL,
    amount       NUMERIC(12,2) NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_expenses_vehicle_id ON expenses(vehicle_id);
