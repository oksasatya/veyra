CREATE TABLE vehicles (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    brand            TEXT NOT NULL,
    model            TEXT NOT NULL,
    year             SMALLINT NOT NULL,
    plate_number     TEXT NOT NULL,
    color            TEXT,
    fuel_type        TEXT NOT NULL,
    current_odometer INTEGER NOT NULL DEFAULT 0,
    notes            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, plate_number)
);

CREATE INDEX idx_vehicles_user_id ON vehicles(user_id);
