use log::{info, error};
use serenity::all::ChannelId;
use serenity::{futures::StreamExt, model::Timestamp};
use sqlx::MySqlPool;

pub async fn scrape_messages(
    ctx: serenity::client::Context,
    _bot: &serenity::model::gateway::Ready,
    channel_id: ChannelId,
    db_pool: &MySqlPool,
    start_date: Timestamp,
    end_date: Timestamp,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("scrape_messages: Starting scrape");

    let mut messages = channel_id.messages_iter(&ctx.http).boxed();

    while let Some(message) = messages.next().await {
        match message {
            Ok(msg) => {
                if msg.timestamp > start_date && msg.timestamp < end_date.into() {
                    // Print the message details
                    info!(
                        "scrape_messages: {}@{}@{}@{}@{}@{}@{:?}",
                        &msg.id,
                        &msg.channel_id,
                        &msg.author.id,
                        &msg.author.name,
                        &msg.timestamp,
                        &msg.content,
                        &msg.author.premium_type
                    );

                    let timestamp_str = msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                    let premium_type_str = format!("{:?}", msg.author.premium_type);

                    // Insert message details into the database
                    let insert_query = "
                        INSERT INTO wdl_database.discord_messages
                        (MessageId, ChannelId, UserId, Name, Content, Timestamp, PremiumType)
                        VALUES (?, ?, ?, ?, ?, ?, ?);
                    ";

                    // Execute the query
                    if let Err(e) = sqlx::query(insert_query)
                        .bind(i64::from(msg.id))
                        .bind(i64::from(msg.channel_id))
                        .bind(i64::from(msg.author.id))
                        .bind(msg.author.name)
                        .bind(msg.content)
                        .bind(timestamp_str)
                        .bind(premium_type_str)
                        .execute(db_pool)
                        .await
                    {
                        error!("Failed to insert message: {}", e);
                        // Continue processing other messages even if one fails
                    }
                }
            }
            Err(e) => {
                error!("Error while fetching a message: {:?}", e);
                // Continue processing other messages
            }
        }
    }
    info!("scrape_messages: Done downloading!");
    Ok(())
}
