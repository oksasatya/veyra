CREATE TABLE reminders (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id    UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    reminder_type TEXT NOT NULL,
    due_date      DATE,
    due_odometer  INTEGER,
    is_completed  BOOLEAN NOT NULL DEFAULT FALSE,
    notes         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reminders_vehicle_id_due ON reminders(vehicle_id, due_date);
