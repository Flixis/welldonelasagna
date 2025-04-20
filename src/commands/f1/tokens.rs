use sqlx::{Error, MySqlPool};

/// Fetches the number of remaining tokens for a given user.
/// Returns Ok(None) if the user is not found in the token table.
pub async fn get_user_tokens(pool: &MySqlPool, user_id: i64) -> Result<Option<i32>, Error> {
    let result = sqlx::query!(
        "SELECT tokens_remaining FROM f1_fantasy_tokens WHERE user_id = ?",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| row.tokens_remaining))
}

/// Consumes one token for the given user if they have tokens remaining.
/// Returns Ok(true) if a token was successfully consumed.
/// Returns Ok(false) if the user has no tokens left or is not found.
pub async fn use_token(pool: &MySqlPool, user_id: i64) -> Result<bool, Error> {
    // Update tokens_remaining and get the number of affected rows
    let result = sqlx::query!(
        "UPDATE f1_fantasy_tokens
            SET tokens_remaining = tokens_remaining - 1
            WHERE user_id = ? AND tokens_remaining > 0",
        user_id
    )
    .execute(pool)
    .await?;
    
    let success = result.rows_affected() > 0;
    
    // If the update was successful, try to log it in the token usage table
    if success {
        // This may fail if the table doesn't exist yet, but we don't want to fail
        // the whole operation - we just want to consume the token.
        let _ = sqlx::query!(
            "INSERT INTO f1_token_usage (user_id, reason) VALUES (?, 'Team swap')",
            user_id
        )
        .execute(pool)
        .await;
    }
    
    Ok(success)
}

/// Retrieves token usage history for a specific user
pub async fn get_user_token_usage(pool: &MySqlPool, user_id: i64) -> Result<Vec<(String, String)>, Error> {
    // Try to get usage history, but return empty vec if table doesn't exist
    let results = match sqlx::query!(
        "SELECT used_at, reason FROM f1_token_usage WHERE user_id = ? ORDER BY used_at DESC",
        user_id
    )
    .fetch_all(pool)
    .await {
        Ok(results) => results,
        Err(_) => return Ok(Vec::new()), // Table likely doesn't exist yet or other error
    };
    
    Ok(results
        .into_iter()
        .map(|row| (
            row.used_at.to_string(),
            row.reason
        ))
        .collect())
}

/// Retrieves all token usage history (admin only)
pub async fn get_all_token_usage(pool: &MySqlPool) -> Result<Vec<(i64, String, String)>, Error> {
    // Try to get all usage history, but return empty vec if table doesn't exist
    let results = match sqlx::query!(
        "SELECT user_id, used_at, reason FROM f1_token_usage ORDER BY used_at DESC"
    )
    .fetch_all(pool)
    .await {
        Ok(results) => results,
        Err(_) => return Ok(Vec::new()), // Table likely doesn't exist yet or other error
    };
    
    Ok(results
        .into_iter()
        .map(|row| (
            row.user_id,
            row.used_at.to_string(),
            row.reason
        ))
        .collect())
}

/// Adds a user to the f1_fantasy_tokens table with the default number of tokens.
/// If the user already exists, this function does nothing.
pub async fn add_user_token(pool: &MySqlPool, user_id: i64) -> Result<(), Error> {
    sqlx::query!(
        "INSERT IGNORE INTO f1_fantasy_tokens (user_id) VALUES (?)",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetches all users and their remaining tokens.
pub async fn get_all_user_tokens(pool: &MySqlPool) -> Result<Vec<(i64, Option<i32>)>, Error> {
    let results = sqlx::query!(
        "SELECT user_id, tokens_remaining FROM f1_fantasy_tokens"
    )
    .fetch_all(pool)
    .await?;

    let mapped_results: Vec<(i64, Option<i32>)> = results
        .into_iter()
        .map(|row| (row.user_id, Some(row.tokens_remaining)))
        .collect();

    Ok(mapped_results)
}
