use clap::Parser;
use serenity::{
    all::ChannelId,
    async_trait,
    model::{channel::Message, gateway::Ready, Timestamp},
    prelude::*,
};
use sqlx::mysql::MySqlPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use commands::{quote, scraper};

mod cli;
mod commands;
mod setup;

const VERSION: &str = env!("CARGO_PKG_VERSION"); //<-- read from cargo.toml

struct Handler {
    db_pool: MySqlPool,
    channel_id: ChannelId,
    counter: Mutex<usize>,
    roll_amount: Option<usize>,
    scraping: bool,
    start_date: Option<Timestamp>,
    end_date: Option<Timestamp>,
}

impl Handler {
    fn new(
        db_pool: MySqlPool,
        channel_id: ChannelId,
        roll_amount: Option<usize>,
        scraping: bool,
        start_date: Option<Timestamp>,
        end_date: Option<Timestamp>,
    ) -> Self {
        Handler {
            db_pool,
            channel_id,
            counter: Mutex::new(0),
            roll_amount,
            scraping,
            start_date,
            end_date
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, bot: Ready) {
        // let start_date: DateTime<Utc> = Utc::now() - Duration::days(1); // 7 days ago
        // // let end_date: DateTime<Utc> = Utc::now(); // now
        // let start_date = Timestamp::parse("2028-01-01T00:00:00Z").unwrap();
        // let end_date = Timestamp::parse("2028-12-31T23:59:59Z").unwrap();

        let start_date = match self.start_date {
            Some(start_date) => start_date,
            None => panic!("Start date not set"),
        };
        

        let end_date = match self.end_date {
            Some(end_date) => end_date,
            None => panic!("Start date not set"),
        };

        println!("Using dates: {start_date} and {end_date}");

        if self.scraping {
            scraper::scrape_messages(
                ctx,
                &bot,
                self.channel_id,
                &self.db_pool,
                start_date,
                end_date,
            )
            .await;
        }
        println!("{} is connected!", bot.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let mut counter = self.counter.lock().await;

        let effective_roll_amount = match self.roll_amount {
            Some(amount) => amount,
            None => 15, // Default value
        };

        quote::roll_quote(
            ctx,
            &msg,
            self.channel_id,
            &mut *counter,
            effective_roll_amount,
            &self.db_pool,
        )
        .await;

        println!("{}: {} @ {}", msg.author, msg.content, msg.timestamp);
    }
}

#[tokio::main]
async fn main() {
    // Generate a random UUID
    let random_uuid = Uuid::new_v4();
    println!("Bot version: {}", VERSION);
    println!("Instance check: {}", random_uuid);

    let cli_args: cli::CliCommands = cli::CliCommands::parse();

    match setup::setup().await {
        Ok((db_pool, discord_token, channel_id)) => {
            // Create an instance of handler and fill its contents
            let handler =
                Handler::new(db_pool, channel_id, cli_args.roll_amount, cli_args.scraping,
                cli_args.start_date, cli_args.end_date);

            let intents = GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT;

            let mut client = Client::builder(&discord_token, intents)
                .event_handler(handler) // Pass the handler instance here
                .await
                .expect("Error creating client");

            if let Err(error) = client.start().await {
                println!("Client error: {:?}", error);
            }
        }
        Err(error) => {
            // Error handling logic
            eprintln!("Failed to set up: {}", error);
        }
    }
}
