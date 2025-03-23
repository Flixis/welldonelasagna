use serenity::model::id::ChannelId;
use sqlx::MySqlPool;
use std::error::Error;
use std::str::FromStr;

pub async fn setup() -> Result<(MySqlPool, String, ChannelId), Box<dyn Error>> {
    dotenv::dotenv().ok();

    // Establish connection to the database
    let database_url = match dotenv::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => return Err("DATABASE_URL is not set in environment variables".into()),
    };
    let db_pool = MySqlPool::connect(&database_url)
        .await
        .map_err(|_| "Failed to connect to the database")?;

    let discord_channel_id = match dotenv::var("DISCORD_CHANNEL_ID") {
        Ok(cid) => cid,
        Err(_) => return Err("DISCORD_CHANNEL_ID is not set in environment variables".into()),
    };
    let channel_id = ChannelId::from_str(&discord_channel_id)
        .map_err(|_| "Failed to parse DISCORD_CHANNEL_ID into a ChannelId")?;

    let discord_token = match dotenv::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(_) => return Err("DISCORD_TOKEN is not set in environment variables".into()),
    };

    Ok((db_pool, discord_token, channel_id))
}
