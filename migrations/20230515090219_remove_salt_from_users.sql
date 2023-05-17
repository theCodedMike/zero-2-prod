-- sqlx migrate add remove_salt_from_users

-- Add migration script here
ALTER TABLE users DROP COLUMN salt;