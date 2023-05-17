-- sqlx migrate add rename_password_column

-- Add migration script here
ALTER TABLE users RENAME password TO password_hash;