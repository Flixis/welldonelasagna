use std::str::FromStr;
use dotenv::dotenv;
use logging_settings::setup_loggers;
use serenity::futures::StreamExt;
use serenity::http::RatelimitInfo;
use serenity::model::Timestamp;
use serenity::{
    async_trait,
    model::{gateway::Ready, id::ChannelId},
    prelude::*,
};
use sqlx::mysql::MySqlPool;


mod logging_settings;

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

    async fn ratelimit(&self, data: RatelimitInfo) {

        log::warn!("ratelimit limit: {} with duration: {:?}", data.limit, data.timeout);

    }

    async fn ready(&self, ctx: Context, bot: Ready) {
        log::info!("{} is connected!", bot.user.name);

        let discord_channel_id = match dotenv::var("DISCORD_CHANNEL_ID") {
            Ok(val) => val,
            Err(_) => {
                log::info!("Missing DISCORD_CHANNEL_ID in environment variable");
                return;
            }
        };

        let channel_id = match ChannelId::from_str(&discord_channel_id) {
            Ok(val) => val,
            Err(_) => {
                log::info!("Couldn't parse DISCORD_CHANNEL_ID for ChannelId");
                return;
            }
        };

        // let start_date: DateTime<Utc> = Utc::now() - Duration::days(1); // 7 days ago
        // let end_date: DateTime<Utc> = Utc::now(); // now
        let start_date = Timestamp::parse("2017-01-01T00:00:00Z").unwrap();
        let end_date = Timestamp::parse("2021-12-31T23:59:59Z").unwrap();

        let mut messages = channel_id.messages_iter(&ctx.http).boxed();

        while let Some(message) = messages.next().await {
            log::info!("Receving message....");
            match message {
                Ok(msg) => {
                    if msg.timestamp > start_date && msg.timestamp < end_date.into() {
                        // Print the message details
                        log::info!(
                            "{}@{}@{}@{}@{}@{}@{:?}",
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
                        let _ = sqlx::query(insert_query)
                            .bind(i64::from(msg.id)) // Assuming msg.id is an ID type that can be converted to i64
                            .bind(i64::from(msg.channel_id))
                            .bind(i64::from(msg.author.id))
                            .bind(msg.author.name)
                            .bind(msg.content)
                            .bind(timestamp_str)
                            .bind(premium_type_str)
                            .execute(&self.db_pool)
                            .await
                            .map_err(|e| log::info!("Failed to insert message: {}", e));
                    }
                }
                Err(why) => log::info!("Error while fetching a message: {:?}", why),
            }
        }
        log::info!("Done downloading!");
    }
}

#[tokio::main]
async fn main() {

    setup_loggers("wdl_discord_scraper".to_string());
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
        log::info!("Client error: {:?}", why);
    }
}
