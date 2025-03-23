use clap::Parser;
use log::{info, warn};
use std::{fs, path::Path, sync::OnceLock};
use toml::Value;
use serenity::{
    all::{ChannelId, Command, CreateCommand},
    async_trait,
    model::{channel::Message, gateway::Ready, Timestamp},
    prelude::*,
};
use sqlx::mysql::MySqlPool;
use tokio::sync::Mutex;

use commands::{quote, scraper, version};

mod cli;
mod commands;
mod logging_settings;
mod setup;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_ID: &str = env!("BUILD_ID");

static ALLOWED_QUOTE_USERS: OnceLock<Vec<i64>> = OnceLock::new();

fn ensure_config_exists() {
    let config_dir = Path::new("config");
    let config_file = config_dir.join("quote_settings.toml");

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(config_dir).expect("Failed to create config directory");
    }

    // Create config file with default values if it doesn't exist
    if !config_file.exists() {
        let default_config = r#"# List of Discord user IDs that can be used for quotes
# Format: Array of integers representing Discord user IDs
allowed_user_ids = [
    121751619149758464,
    98443943032684544,
    164878773349646336,
    248870522975289344,
    181824467851280395,
    168785146206617601,
    282681436132212736,
    95565218498748416,
    1092454499236462783,
    243785081167151104,
]"#;
        fs::write(config_file, default_config).expect("Failed to create default config file");
    }
}

fn load_allowed_user_ids() -> Vec<i64> {
    match fs::read_to_string("config/quote_settings.toml") {
        Ok(content) => {
            match content.parse::<Value>() {
                Ok(value) => {
                    if let Some(array) = value.get("allowed_user_ids").and_then(|v| v.as_array()) {
                        array.iter()
                            .filter_map(|v| v.as_integer().map(|i| i as i64))
                            .collect()
                    } else {
                        warn!("No allowed_user_ids found in config, using empty list");
                        Vec::new()
                    }
                }
                Err(e) => {
                    warn!("Failed to parse config file: {}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            warn!("Failed to read config file: {}", e);
            Vec::new()
        }
    }
}

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
            end_date,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, bot: Ready) {
        // Register commands
        let commands = vec![
            CreateCommand::new("guessquote")
                .description("Start a game where you have to guess who said a quote"),
            quote::register(),
            version::register(),
        ];

        Command::set_global_commands(&ctx.http, commands)
            .await
            .expect("Failed to create commands");

        if self.scraping {
            // let start_date: DateTime<Utc> = Utc::now() - Duration::days(1); // 7 days ago
            // let end_date: DateTime<Utc> = Utc::now(); // now
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

            info!("Using dates: {start_date} and {end_date}");

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
        info!("{} is connected!", bot.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: serenity::model::application::Interaction) {
        if let serenity::model::application::Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "guessquote" => {
                    quote::guess_quote(ctx, &command, &self.db_pool).await;
                }
                "scoreboard" => {
                    quote::show_scoreboard(ctx, &command, &self.db_pool).await;
                }
                "version" => {
                    version::show_version(ctx, &command).await;
                }
                _ => {}
            }
        }
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

        info!("{}: {} @ {}", msg.author, msg.content, msg.timestamp);
    }
}

#[tokio::main]
async fn main() {
    logging_settings::setup_loggers();
    
    // Ensure config exists and load allowed users
    ensure_config_exists();
    let allowed_users = load_allowed_user_ids();
    ALLOWED_QUOTE_USERS.set(allowed_users).expect("Failed to set allowed users");
    let cli_args: cli::CliCommands = cli::CliCommands::parse();

    // Generate a random UUID
    info!("Bot version: {} (build: {})", VERSION, BUILD_ID);

    match setup::setup().await {
        Ok((db_pool, discord_token, channel_id)) => {
            // Create an instance of handler and fill its contents
            let handler = Handler::new(
                db_pool,
                channel_id,
                cli_args.roll_amount,
                cli_args.scraping,
                cli_args.start_date,
                cli_args.end_date,
            );

            let intents = GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT
                | GatewayIntents::GUILD_MESSAGE_REACTIONS;

            let mut client = Client::builder(&discord_token, intents)
                .event_handler(handler) // Pass the handler instance here
                .await
                .expect("Error creating client");

            if let Err(error) = client.start().await {
                info!("Client error: {:?}", error);
            }
        }
        Err(error) => {
            warn!("Failed to set up: {}", error);
        }
    }
}
