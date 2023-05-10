-- 1: execute command 'sqlx migrate add create_subscriptions_table' at terminal
-- 2: as a result of step 1, {timestamp}_create_subscriptions_table.sql file was generated at migrations directory
-- 3: add sql script(create table) at {timestamp}_create_subscriptions_table.sql
-- 4: execute sql script, a table would be generated at database

-- Add migration script here
-- Create Subscriptions Table
CREATE TABLE subscriptions
(
    id            uuid        NOT NULL,
    PRIMARY KEY (id),
    email         TEXT        NOT NULL UNIQUE,
    name          TEXT        NOT NULL,
    subscribed_at timestamptz NOT NULL
);