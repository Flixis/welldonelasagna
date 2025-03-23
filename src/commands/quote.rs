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
use crate::ALLOWED_QUOTE_USERS;

pub fn register() -> CreateCommand {
    CreateCommand::new("scoreboard")
        .description("View the guessquote game scoreboard")
}

pub async fn show_scoreboard(
    ctx: serenity::client::Context,
    command: &CommandInteraction,
    db_pool: &MySqlPool,
) {
    info!("Fetching scoreboard...");
    let query = "
    WITH latest_names AS (
        SELECT UserId,
               Name,
               ROW_NUMBER() OVER (PARTITION BY UserId ORDER BY Timestamp DESC) as rn
        FROM wdl_database.discord_messages
    )
    SELECT qs.user_id, 
           COALESCE(ln.Name, CONVERT(qs.user_id, CHAR CHARACTER SET utf8mb4)) as Name,
           qs.correct_guesses, 
           qs.total_attempts, 
           qs.points,
           qs.current_streak,
           qs.best_streak,
           CAST((qs.correct_guesses * 100.0 / qs.total_attempts) AS DOUBLE) as accuracy
    FROM wdl_database.quote_scores qs
    LEFT JOIN latest_names ln ON ln.UserId = qs.user_id AND ln.rn = 1
    ORDER BY qs.points DESC, accuracy DESC
    LIMIT 10;
    ";

    let result = sqlx::query_as::<_, (i64, String, i32, i32, i32, i32, i32, Option<f64>)>(query)
        .fetch_all(db_pool)
        .await;

    match result {
        Ok(scores) => {
            info!("Found {} players on scoreboard", scores.len());
            let mut scoreboard = String::from("🏆 **GuessQuote Leaderboard** 🏆\n\n");
            for (index, (user_id, name, correct, total, points, current_streak, best_streak, accuracy)) in scores.iter().enumerate() {
                let accuracy_value = accuracy.unwrap_or(0.0);
                info!("Rank {}: {} (ID: {}) - {} points, {}/{} correct, streak: {}/{} ({}% accuracy)",
                    index + 1, name, user_id, points, correct, total, current_streak, best_streak, accuracy_value.round());
                scoreboard.push_str(&format!(
                    "{}. {} - {} points, {} correct out of {} attempts ({}% accuracy) | Streak: {} 🔥 (Best: {})\n",
                    index + 1,
                    name,
                    points,
                    correct,
                    total,
                    accuracy_value.round(),
                    current_streak,
                    best_streak
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
    // Get allowed user IDs from static
    let empty_vec = Vec::new();
    let allowed_users = ALLOWED_QUOTE_USERS.get().unwrap_or(&empty_vec);
    
    info!("Starting new quote game. Allowed users: {:?}", allowed_users);
    
    // Build query
    let base_query = "SELECT Id, UserId, Name, Content, Timestamp 
         FROM wdl_database.discord_messages
         WHERE CHAR_LENGTH(Content) >= 20 ";
    let mut query_builder = sqlx::QueryBuilder::new(base_query);

    if !allowed_users.is_empty() {
        query_builder.push("AND UserId IN (");
        let mut separated = query_builder.separated(", ");
        for &id in allowed_users.iter() {
            separated.push_bind(id);
        }
        separated.push_unseparated(") ");
    }

    query_builder.push("ORDER BY RAND() LIMIT 1");

    let result = query_builder.build_query_as::<(i64, i64, String, String, chrono::DateTime<Utc>)>()
        .fetch_one(db_pool)
        .await;

    match result {
        Ok(row) => {
            // Log the correct answer for debugging
            info!("Selected quote - ID: {}, User: {} (ID: {}), Content: {:?}, Time: {}", 
                row.0, row.2, row.1, row.3, row.4);
            
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
                        info!("Skipping duplicate guess from user {}", guess.author.id);
                        continue;
                    }
                    
                    let correct_user_id = row.1.to_string();
                    let correct_name = row.2.to_lowercase(); // Get the correct username
                    let message_content = guess.content.to_lowercase();
                    
                    info!("Processing guess from {} - content: {:?}", guess.author.id, message_content);
                    
                    // Check if the guess is correct
                    let has_correct_mention = guess.mentions.iter().any(|user| user.id.to_string() == correct_user_id);
                    let contains_correct_name = message_content.contains(&correct_name);
                    let is_correct = has_correct_mention || contains_correct_name;
                    
                    info!("Guess analysis - has_mention: {}, has_name: {}, is_correct: {}", 
                        has_correct_mention, contains_correct_name, is_correct);
                    
                    // Store the guess result
                    guesses.push((guess.author.id, is_correct));
                    
                    // Only mark user as having guessed if they got it right
                    if is_correct {
                        info!("Correct guess from user {}", guess.author.id);
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
                info!("No guesses received for this quote");
                if let Err(why) = channel_id.say(&ctx.http, response).await {
                    warn!("Error sending response: {why}");
                }
                return;
            }
            
            info!("Processing {} guesses", guesses.len());
            
            // Collect results before updating scores
            let mut correct_guesses = Vec::new();
            let mut incorrect_guesses = Vec::new();
            
            // Process all guesses and update scores
            for &(user_id, is_correct) in guesses.iter() {
                // Calculate points based on response time (max 30 seconds)
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let points = if is_correct {
                    // Points formula: max 100 points at 0 seconds, decreasing to 10 points at 30 seconds
                    let time_points = ((30.0 - elapsed_secs) / 30.0 * 90.0 + 10.0) as i32;
                    let final_points = time_points.max(10); // Ensure minimum 10 points for correct answer
                    info!(
                        "Points calculation for user {}: elapsed_time={:.2}s, raw_points={}, final_points={}",
                        user_id, elapsed_secs, time_points, final_points
                    );
                    final_points
                } else {
                    info!("Subtracting points from user {} (incorrect guess)", user_id);
                    -5 // Subtract 5 points for incorrect answers
                };

                // Update scores in database
                // Calculate streak bonus (5 points per streak level, max 25 bonus points)
                let streak_bonus = if is_correct {
                    let current_streak = sqlx::query_as::<_, (i32,)>("SELECT current_streak FROM wdl_database.quote_scores WHERE user_id = ?")
                        .bind(user_id.to_string().parse::<i64>().unwrap())
                        .fetch_optional(db_pool)
                        .await
                        .map(|r| r.map(|s| s.0).unwrap_or(0))
                        .unwrap_or(0);
                    (current_streak.min(5) * 5) as i32
                } else {
                    0
                };

                let final_points = points + streak_bonus;
                info!("Points breakdown - base: {}, streak_bonus: {}, final: {}", points, streak_bonus, final_points);

                // Use separate queries for correct/incorrect to avoid string formatting
                let update_query = if is_correct {
                    "INSERT INTO wdl_database.quote_scores (user_id, correct_guesses, total_attempts, points)
                     VALUES (?, ?, 1, ?)
                     ON DUPLICATE KEY UPDATE 
                     correct_guesses = correct_guesses + VALUES(correct_guesses),
                     total_attempts = total_attempts + 1,
                     points = points + ?,
                     current_streak = current_streak + 1,
                     best_streak = GREATEST(best_streak, current_streak + 1)"
                } else {
                    "INSERT INTO wdl_database.quote_scores (user_id, correct_guesses, total_attempts, points)
                     VALUES (?, ?, 1, ?)
                     ON DUPLICATE KEY UPDATE 
                     correct_guesses = correct_guesses + VALUES(correct_guesses),
                     total_attempts = total_attempts + 1,
                     points = points + ?,
                     current_streak = 0"
                };

                info!("Updating database for user {} - is_correct: {}, points: {}", user_id, is_correct, points);

                if let Err(e) = sqlx::query(update_query)
                    .bind(user_id.to_string().parse::<i64>().unwrap())
                    .bind(if is_correct { 1 } else { 0 })
                    .bind(final_points)
                    .bind(final_points)
                    .execute(db_pool)
                    .await
                {
                    warn!("Failed to update score: {}", e);
                }

                // Get updated stats for the user
                let stats_query = "
                    SELECT correct_guesses, total_attempts, points, current_streak, best_streak,
                           CAST((correct_guesses * 100.0 / total_attempts) AS DOUBLE) as accuracy
                    FROM wdl_database.quote_scores
                    WHERE user_id = ?";

                let stats = sqlx::query_as::<_, (i32, i32, i32, i32, i32, f64)>(stats_query)
                    .bind(user_id.to_string().parse::<i64>().unwrap())
                    .fetch_one(db_pool)
                    .await;

                let user_result = match stats {
                    Ok((correct, total, _total_points, current_streak, best_streak, accuracy)) => {
                        let streak_text = if current_streak > 0 {
                            format!(" | 🔥 Streak: {} (Best: {})", current_streak, best_streak)
                        } else {
                            String::new()
                        };
                        
                        info!("Updated stats for user {}: {}/{} correct, round points: {} (streak: {}/{}), {}% accuracy",
                            user_id, correct, total, final_points, current_streak, best_streak, accuracy.round());
                        format!(
                            "<@{}> - {} points! {} correct out of {} attempts ({}% accuracy){}",
                            user_id,
                            if final_points >= 0 { format!("+{}", final_points) } else { final_points.to_string() },
                            correct,
                            total,
                            accuracy.round(),
                            streak_text
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

        let rand = rand::rng().random_range(0..100);
        info!("rand generated {:?}", rand);

        if rand < 1 {
            // Get allowed user IDs from static
            let empty_vec = Vec::new();
            let allowed_users = ALLOWED_QUOTE_USERS.get().unwrap_or(&empty_vec);
            
            // Build query
            let base_query = "SELECT Id, UserId, Name, Content, Timestamp 
                 FROM wdl_database.discord_messages
                 WHERE CHAR_LENGTH(Content) >= 1 ";
            let mut query_builder = sqlx::QueryBuilder::new(base_query);

            if !allowed_users.is_empty() {
                query_builder.push("AND UserId IN (");
                let mut separated = query_builder.separated(", ");
                for &id in allowed_users.iter() {
                    separated.push_bind(id);
                }
                separated.push_unseparated(") ");
            }

            query_builder.push("ORDER BY RAND() LIMIT 1");

            let result = query_builder.build_query_as::<(i64, i64, String, String, chrono::DateTime<Utc>)>()
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
                if let Err(why) = channel_id
                    .say(
                        &ctx.http,
                        &format!(
                            "> ** <@{}> on {} at {}:**\n> \n> _'{}'_",
                            row.1, timestamp[0], timestamp[1], row.3
                        ),
                    )
                    .await
                {
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
