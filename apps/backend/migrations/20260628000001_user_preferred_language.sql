-- Adds the user's preferred UI/notification language. Static UI copy is owned and
-- localized by the client; this column exists so server-generated content (future
-- maintenance-reminder notifications, emails) can be rendered in the user's language
-- when no client is in the loop. Allowed values mirror the domain `Language` enum.
ALTER TABLE users
    ADD COLUMN preferred_language TEXT NOT NULL DEFAULT 'en'
        CHECK (preferred_language IN ('en', 'id'));
