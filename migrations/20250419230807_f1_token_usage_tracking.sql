-- Add migration script here
-- Create table to track F1 fantasy token usage
CREATE TABLE f1_token_usage (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    used_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason VARCHAR(255) NOT NULL DEFAULT 'Team swap',
    FOREIGN KEY (user_id) REFERENCES f1_fantasy_tokens(user_id) ON DELETE CASCADE
);
