use std::str::FromStr;

use dotenv::dotenv;
use serenity::futures::StreamExt;
use serenity::model::Timestamp;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use sqlx::mysql::MySqlPool;

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
    async fn ready(&self, ctx: Context, bot: Ready) {
        println!("{} is connected!", bot.user.name);

        // Parse the DISCORD_CHANNEL_ID from the environment variable
        let discord_channel_id = dotenv::var("DISCORD_CHANNEL_ID")
            .expect("Missing DISCORD_CHANNEL_ID in environment variable");
        let channel_id = ChannelId::from_str(discord_channel_id.as_str())
            .expect("couldn't parse DISCORD_CHANNEL_ID in for ChannelId");

        // let start_date: DateTime<Utc> = Utc::now() - Duration::days(1); // 7 days ago
        // let end_date: DateTime<Utc> = Utc::now(); // now
        let start_date = Timestamp::parse("2023-01-01T00:00:00Z").unwrap();
        let end_date = Timestamp::parse("2023-12-31T23:59:59Z").unwrap();


        let mut messages = channel_id.messages_iter(&ctx.http).boxed();

        while let Some(message) = messages.next().await {
            match message {
                Ok(msg) => {
                    if msg.timestamp > start_date && msg.timestamp < end_date.into() {
                        // Print the message details
                        println!(
                            "{}@{}@{}@{}@{}@{:?}",
                            &msg.id,
                            &msg.channel_id,
                            &msg.author.id,
                            &msg.timestamp,
                            &msg.content,
                            &msg.author.premium_type
                        );

                        let timestamp_str = msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                        let premium_type_str = format!("{:?}", msg.author.premium_type);

                        // Insert message details into the database
                        let insert_query = "
                            INSERT INTO wdl_database.discord_messages 
                            (MessageId, ChannelId, UserId, Content, Timestamp, PremiumType) 
                            VALUES (?, ?, ?, ?, ?, ?);
                        ";

                        // Execute the query
                        let _ = sqlx::query(insert_query)
                            .bind(i64::from(msg.id)) // Assuming msg.id is an ID type that can be converted to i64
                            .bind(i64::from(msg.channel_id))
                            .bind(i64::from(msg.author.id))
                            .bind(&msg.content)
                            .bind(timestamp_str)
                            .bind(premium_type_str)
                            .execute(&self.db_pool)
                            .await
                            .map_err(|e| println!("Failed to insert message: {}", e));

                    }
                }
                Err(why) => println!("Error while fetching a message: {:?}", why),
            }
        }
        println!("Done downloading!");
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        println!("{}: {} @ {}", msg.author, msg.content, msg.timestamp);
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
