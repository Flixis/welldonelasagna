USE wdl_database;

-- Create users table to track unique Discord users
CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT PRIMARY KEY,
    name VARCHAR(255)
);

-- Insert unique users from discord_messages
INSERT IGNORE INTO users (user_id, name)
SELECT DISTINCT UserId, Name
FROM discord_messages;

-- Add index to UserId in discord_messages table
ALTER TABLE discord_messages
ADD INDEX idx_user_id (UserId);

-- Create scores table referencing users table
CREATE TABLE IF NOT EXISTS quote_scores (
    user_id BIGINT PRIMARY KEY,
    correct_guesses INT NOT NULL DEFAULT 0,
    total_attempts INT NOT NULL DEFAULT 0,
    CONSTRAINT fk_user_id FOREIGN KEY (user_id) REFERENCES users(user_id)
);
