use chrono::Utc;
use dotenv::dotenv;
use rand::Rng;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use sqlx::mysql::MySqlPool;
use std::str::FromStr;

struct Handler {
    db_pool: MySqlPool,
}

// Initialize the Handler with a database connection pool
impl Handler {
    fn new(db_pool: MySqlPool) -> Self {
        Handler { db_pool }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, bot: Ready) {
        println!("{} is connected!", bot.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
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
        
        let rand = rand::thread_rng().gen_range(0..1000);
        println!("rand generated {:?}", rand);
        if  rand < 1 { //%0.1 chance happening
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
                    .fetch_one(&self.db_pool)
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
        } else {
            println!("{}: {} @ {}", msg.author, msg.content, msg.timestamp);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Establish connection to the database
    let database_url =
        dotenv::var("DATABASE_URL").expect("Missing DATABASE_URL in environment variable");
    let db_pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Create an instance of Handler with the database pool
    let handler = Handler::new(db_pool);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let discord_token =
        dotenv::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN in environment variable");

    let mut client = Client::builder(&discord_token, intents)
        .event_handler(handler) // Pass the handler instance here
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
