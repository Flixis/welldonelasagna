use chrono::Utc;
use rand::Rng;
use serenity::all::ChannelId;
use sqlx::MySqlPool;
use std::str::FromStr;

pub async fn roll_quote(
    ctx: serenity::client::Context,
    msg: serenity::model::channel::Message,
    counter: &mut usize,
    roll_amount: usize,
    db_pool: &MySqlPool,
) {
    let discord_channel_id = match dotenv::var("DISCORD_CHANNEL_ID") {
        Ok(val) => val,
        Err(_) => {
            println!("Missing DISCORD_CHANNEL_ID in environment variable");
            return;
        }
    };
    let channel_id = match ChannelId::from_str(&discord_channel_id) {
        Ok(val) => val,
        Err(_) => {
            println!("Couldn't parse DISCORD_CHANNEL_ID for ChannelId");
            return;
        }
    };

    println!("Connected to {:?}", channel_id);

    *counter += 1; // Increment counter
    println!("counter at: {:?}", counter);
    if *counter >= roll_amount {
        *counter = 0; // Reset the counter

        let rand = rand::thread_rng().gen_range(0..100);
        println!("rand generated {:?}", rand);

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
                    println!(
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
                        eprintln!("Something went wrong: {why}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute query: {}", e);
                }
            }
        }
    }
    println!("{}: {} @ {}", msg.author, msg.content, msg.timestamp);
}
