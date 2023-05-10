-- sqlx migrate add add_status_to_subscriptions

-- Add migration script here
ALTER TABLE subscriptions ADD COLUMN status TEXT NULL ;