-- Add streak column to quote_scores table
ALTER TABLE wdl_database.quote_scores
ADD COLUMN current_streak INT NOT NULL DEFAULT 0,
ADD COLUMN best_streak INT NOT NULL DEFAULT 0;

-- Initialize best_streak to match any existing streaks
UPDATE wdl_database.quote_scores
SET best_streak = current_streak;
