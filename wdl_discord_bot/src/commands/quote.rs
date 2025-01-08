use chrono::Utc;
use log::{info, warn};
use rand::Rng;
use serenity::all::{ChannelId, CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage};
use sqlx::MySqlPool;
use std::time::Duration;

pub async fn guess_quote(
    ctx: serenity::client::Context,
    command: &CommandInteraction,
    db_pool: &MySqlPool,
) {
    let query = "
    SELECT Id, UserId, Name, Content, Timestamp 
    FROM wdl_database.discord_messages
    WHERE CHAR_LENGTH(Content) >= 20
    ORDER BY RAND()
    LIMIT 1;            
    ";

    // Execute the query
    let result = sqlx::query_as::<_, (i64, i64, String, String, chrono::DateTime<Utc>)>(query)
        .fetch_one(db_pool)
        .await;

    match result {
        Ok(row) => {
            let quote_message = format!(
                "**Guess who said this quote:**\n\n> _{}_\n\nYou have 30 seconds to guess! Mention the user with @username.",
                row.3
            );

            // Send the initial message
            if let Err(why) = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(&quote_message),
                    ),
                )
                .await
            {
                warn!("Error sending quote: {why}");
                return;
            }

            // Get the channel ID from the command
            let channel_id = command.channel_id;

            // Create a collector for messages
            let message_collector = channel_id
                .await_reply(&ctx.shard)
                .timeout(Duration::from_secs(30))
                .author_id(command.user.id)
                .await;

            match message_collector {
                Some(guess) => {
                    let correct_user_id = row.1.to_string();
                    if guess.mentions.iter().any(|user| user.id.to_string() == correct_user_id) {
                        let response = format!("🎉 Correct! The quote was indeed from <@{}>!", row.1);
                        if let Err(why) = guess.reply(&ctx.http, &response).await {
                            warn!("Error sending response: {why}");
                        }
                    } else {
                        let response = format!("❌ Wrong! The quote was actually from <@{}>.", row.1);
                        if let Err(why) = guess.reply(&ctx.http, &response).await {
                            warn!("Error sending response: {why}");
                        }
                    }
                }
                None => {
                    let timeout_msg = format!("Time's up! The quote was from <@{}>.", row.1);
                    if let Err(why) = channel_id.say(&ctx.http, timeout_msg).await {
                        warn!("Error sending timeout message: {why}");
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to execute query: {}", e);
            if let Err(why) = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Sorry, I couldn't fetch a quote right now."),
                    ),
                )
                .await
            {
                warn!("Error sending error message: {why}");
            }
        }
    }
}

pub async fn roll_quote(
    ctx: serenity::client::Context,
    _msg: &serenity::model::channel::Message,
    channel_id: ChannelId,
    counter: &mut usize,
    roll_amount: usize,
    db_pool: &MySqlPool,
) {
    info!("Connected to {:?}", channel_id);

    *counter += 1; // Increment counter
    info!("counter at: {:?}", counter);
    if *counter >= roll_amount {
        *counter = 0; // Reset the counter

        let rand = rand::thread_rng().gen_range(0..100);
        info!("rand generated {:?}", rand);

        if rand < 1 {
            let query = "
            SELECT Id, UserId, Name, Content, Timestamp 
            FROM wdl_database.discord_messages
            WHERE CHAR_LENGTH(Content) >= 1
            ORDER BY RAND()
            LIMIT 1;            
            ";

            // Execute the query
            let result =
                sqlx::query_as::<_, (i64, i64, String, String, chrono::DateTime<Utc>)>(query)
                    .fetch_one(db_pool)
                    .await;

            match result {
                Ok(row) => {
                    // Print the data
                    info!(
                        "Id: {}, UserId: {}, Name: {}, Content: {}, Timestamp: {}",
                        row.0, row.1, row.2, row.3, row.4
                    );

                    // Store the string in a variable
                    let timestamp_string = row.4.to_string();
                    // Now split the string and collect into Vec
                    let timestamp: Vec<_> = timestamp_string.split(" ").collect();
                    let message = format!(
                        "> ** <@{}> on {} at {}:**\n> \n> _'{}'_",
                        row.1, timestamp[0], timestamp[1], row.3
                    );

                    if let Err(why) = channel_id.say(&ctx.http, message).await {
                        warn!("Something went wrong: {why}");
                    }
                }
                Err(e) => {
                    warn!("Failed to execute query: {}", e);
                }
            }
        }
    }
}
