-- Add migration script here
CREATE TABLE f1_fantasy_tokens (
    user_id BIGINT PRIMARY KEY,
    tokens_remaining INTEGER NOT NULL DEFAULT 2
);
