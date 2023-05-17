-- sqlx migrate add add_salt_to_users

-- Add migration script here
ALTER TABLE users ADD COLUMN salt TEXT NOT NULL ;