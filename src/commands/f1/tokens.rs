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
    let result = sqlx::query!(
        "UPDATE f1_fantasy_tokens
            SET tokens_remaining = tokens_remaining - 1
            WHERE user_id = ? AND tokens_remaining > 0",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

/// Adds a user to the f1_fantasy_tokens table with the default number of tokens.
/// If the user already exists, this function does nothing.
pub async fn add_user_token(pool: &MySqlPool, user_id: i64) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO f1_fantasy_tokens (user_id) VALUES (?)",
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
