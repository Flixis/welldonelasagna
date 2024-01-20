use clap::Parser;
use dotenv::dotenv;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use sqlx::mysql::MySqlPool;
use tokio::sync::Mutex;
use uuid::Uuid;

mod cli;
mod quote;

struct Handler {
    db_pool: MySqlPool,
    counter: Mutex<usize>,
    roll_amount: Option<usize>,
}

impl Handler {
    fn new(db_pool: MySqlPool, roll_amount: Option<usize>) -> Self {
        Handler {
            db_pool,
            counter: Mutex::new(0),
            roll_amount,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, bot: Ready) {
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
            msg,
            &mut *counter,
            effective_roll_amount,
            &self.db_pool,
        )
        .await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Generate a random UUID
    let random_uuid = Uuid::new_v4();
    println!("Version check: {}", random_uuid);

    let cli_args: cli::CliCommands = cli::CliCommands::parse();

    // Establish connection to the database
    let database_url =
        dotenv::var("DATABASE_URL").expect("Missing DATABASE_URL in environment variable");
    let db_pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Create an instance of handler and fill its contents
    let handler = Handler::new(db_pool, cli_args.roll_amount);

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
