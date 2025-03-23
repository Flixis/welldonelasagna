-- Add points column to quote_scores table
ALTER TABLE wdl_database.quote_scores
ADD COLUMN points INT NOT NULL DEFAULT 0;

-- Update existing records to have points equal to correct guesses (for backward compatibility)
UPDATE wdl_database.quote_scores
SET points = correct_guesses;
