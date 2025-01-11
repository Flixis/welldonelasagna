use chrono::Utc;
use log::{info, warn};
use rand::Rng;
use serenity::all::{
    ChannelId, CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateCommand,
};
use sqlx::MySqlPool;
use std::time::Duration;
use std::collections::HashSet;

pub fn register() -> CreateCommand {
    CreateCommand::new("scoreboard")
        .description("View the guessquote game scoreboard")
}

pub async fn show_scoreboard(
    ctx: serenity::client::Context,
    command: &CommandInteraction,
    db_pool: &MySqlPool,
) {
    let query = "
    SELECT user_id, correct_guesses, total_attempts,
           CAST((correct_guesses * 100.0 / total_attempts) AS DOUBLE) as accuracy
    FROM wdl_database.quote_scores
    ORDER BY accuracy DESC, correct_guesses DESC
    LIMIT 10;
    ";

    let result = sqlx::query_as::<_, (i64, i32, i32, f64)>(query)
        .fetch_all(db_pool)
        .await;

    match result {
        Ok(scores) => {
            let mut scoreboard = String::from("🏆 **GuessQuote Leaderboard** 🏆\n\n");
            for (index, (user_id, correct, total, accuracy)) in scores.iter().enumerate() {
                scoreboard.push_str(&format!(
                    "{}. <@{}> - {} correct out of {} attempts ({}% accuracy)\n",
                    index + 1,
                    user_id,
                    correct,
                    total,
                    accuracy.round()
                ));
            }

            if scores.is_empty() {
                scoreboard.push_str("No scores recorded yet! Start playing with /guessquote");
            }

            if let Err(why) = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(&scoreboard),
                    ),
                )
                .await
            {
                warn!("Error sending scoreboard: {why}");
            }
        }
        Err(e) => {
            warn!("Failed to fetch scoreboard: {}", e);
            if let Err(why) = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Sorry, I couldn't fetch the scoreboard right now."),
                    ),
                )
                .await
            {
                warn!("Error sending error message: {why}");
            }
        }
    }
}

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
            // Log the correct answer for debugging
            info!("GuessQuote Answer - User: {} (ID: {})", row.2, row.1);
            
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

            // Collect all guesses for 30 seconds
            let mut guesses = Vec::new();
            let mut guessed_users = HashSet::new();
            
            let start_time = std::time::Instant::now();
            while start_time.elapsed() < Duration::from_secs(30) {
                if let Some(guess) = channel_id
                    .await_reply(&ctx.shard)
                    .timeout(Duration::from_secs(1))
                    .await 
                {
                    // Skip if user has already guessed
                    if guessed_users.contains(&guess.author.id) {
                        continue;
                    }
                    
                    let correct_user_id = row.1.to_string();
                    let correct_name = row.2.to_lowercase(); // Get the correct username
                    let message_content = guess.content.to_lowercase();
                    
                    // Check if the guess is correct
                    let has_correct_mention = guess.mentions.iter().any(|user| user.id.to_string() == correct_user_id);
                    let contains_correct_name = message_content.contains(&correct_name);
                    let is_correct = has_correct_mention || contains_correct_name;
                    
                    // Store the guess result
                    guesses.push((guess.author.id, is_correct));
                    
                    // Only mark user as having guessed if they got it right
                    if is_correct {
                        guessed_users.insert(guess.author.id);
                    }
                }
            }

            let mut response = String::new();
            
            // First, show who said the quote with message link
            response.push_str(&format!("Time's up! The quote was from {} on {} at {}\n\n", 
                row.2, row.4.format("%Y-%m-%d"), row.4.format("%H:%M:%S")));

            // Handle no guesses case early
            if guesses.is_empty() {
                if let Err(why) = channel_id.say(&ctx.http, response).await {
                    warn!("Error sending response: {why}");
                }
                return;
            }
            
            // Collect results before updating scores
            let mut correct_guesses = Vec::new();
            let mut incorrect_guesses = Vec::new();
            
            // Process all guesses and update scores
            for &(user_id, is_correct) in guesses.iter() {
                // Update scores in database
                let update_query = if is_correct {
                    "INSERT INTO wdl_database.quote_scores (user_id, correct_guesses, total_attempts)
                     VALUES (?, 1, 1)
                     ON DUPLICATE KEY UPDATE 
                     correct_guesses = correct_guesses + 1,
                     total_attempts = total_attempts + 1"
                } else {
                    "INSERT INTO wdl_database.quote_scores (user_id, correct_guesses, total_attempts)
                     VALUES (?, 0, 1)
                     ON DUPLICATE KEY UPDATE 
                     total_attempts = total_attempts + 1"
                };

                if let Err(e) = sqlx::query(update_query)
                    .bind(user_id.to_string().parse::<i64>().unwrap())
                    .execute(db_pool)
                    .await
                {
                    warn!("Failed to update score: {}", e);
                }

                // Get updated stats for the user
                let stats_query = "
                    SELECT correct_guesses, total_attempts,
                           CAST((correct_guesses * 100.0 / total_attempts) AS DOUBLE) as accuracy
                    FROM wdl_database.quote_scores
                    WHERE user_id = ?";

                let stats = sqlx::query_as::<_, (i32, i32, f64)>(stats_query)
                    .bind(user_id.to_string().parse::<i64>().unwrap())
                    .fetch_one(db_pool)
                    .await;

                let user_result = match stats {
                    Ok((correct, total, accuracy)) => {
                        format!(
                            "<@{}> - {} correct out of {} attempts ({}% accuracy)",
                            user_id,
                            correct,
                            total,
                            accuracy.round()
                        )
                    }
                    Err(e) => {
                        warn!("Failed to fetch user stats: {}", e);
                        format!("<@{}>", user_id)
                    }
                };

                if is_correct {
                    correct_guesses.push(user_result);
                } else {
                    incorrect_guesses.push(user_result);
                }
            }

            // Add correct guesses to response
            if !correct_guesses.is_empty() {
                response.push_str("🎉 **Correct Guesses:**\n");
                for guess in correct_guesses {
                    response.push_str(&format!("✅ {}\n", guess));
                }
                response.push_str("\n");
            }

            // Add incorrect guesses to response
            if !incorrect_guesses.is_empty() {
                response.push_str("❌ **Incorrect Guesses:**\n");
                for guess in incorrect_guesses {
                    response.push_str(&format!("❌ {}\n", guess));
                }
            }

            if let Err(why) = channel_id.say(&ctx.http, response).await {
                warn!("Error sending response: {why}");
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
