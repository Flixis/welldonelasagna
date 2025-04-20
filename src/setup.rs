use serenity::model::id::ChannelId;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use std::error::Error;
use std::str::FromStr;
use std::fs;
use serde::Deserialize;
use std::path::Path;
use log::warn;
use std::time::Duration;

#[derive(Deserialize, Debug)]
pub struct BotConfig {
    pub discord: DiscordConfig,
    pub database: DatabaseConfig,
    pub f1: F1Config,
    #[serde(default)]
    pub debug: DebugConfig,
}

#[derive(Deserialize, Debug)]
pub struct DiscordConfig {
    pub token: String,
    pub channel_id: String,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct F1Config {
    pub role_id: String,
    pub channel_id: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct DebugConfig {
    #[serde(default)]
    pub channel_id: String,
}

impl BotConfig {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Use environment variables as backup
        dotenv::dotenv().ok();
        
        // Try to read from config file first
        let config_path = "config/bot_settings.toml";
        
        if Path::new(config_path).exists() {
            let config_content = fs::read_to_string(config_path)?;
            let config: BotConfig = toml::from_str(&config_content)?;
            
            // Validate config
            if config.discord.token.is_empty() || 
               config.discord.channel_id.is_empty() || 
               config.database.url.is_empty() {
                // Fall back to environment variables with a warning
                warn!("Config file exists but has empty values, falling back to environment variables");
                Self::from_env()
            } else {
                Ok(config)
            }
        } else {
            // Fall back to environment variables
            warn!("Config file not found, falling back to environment variables");
            Self::from_env()
        }
    }
    
    fn from_env() -> Result<Self, Box<dyn Error>> {
        let discord_token = dotenv::var("DISCORD_TOKEN")
            .map_err(|_| "DISCORD_TOKEN is not set in environment variables")?;
            
        let discord_channel_id = dotenv::var("DISCORD_CHANNEL_ID")
            .map_err(|_| "DISCORD_CHANNEL_ID is not set in environment variables")?;
            
        let database_url = dotenv::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is not set in environment variables")?;
            
        // F1 role ID defaults to a specific value if not specified
        let f1_role_id = dotenv::var("F1_ROLE_ID")
            .unwrap_or_else(|_| "1348688949262024866".to_string());
            
        // F1 channel defaults to main channel if not specified
        let f1_channel_id = dotenv::var("F1_CHANNEL_ID")
            .unwrap_or_else(|_| discord_channel_id.clone());
            
        // Debug channel defaults to main channel if not specified
        let debug_channel_id = dotenv::var("DEBUG_CHANNEL_ID")
            .unwrap_or_else(|_| discord_channel_id.clone());
            
        Ok(BotConfig {
            discord: DiscordConfig {
                token: discord_token,
                channel_id: discord_channel_id,
            },
            database: DatabaseConfig {
                url: database_url,
            },
            f1: F1Config {
                role_id: f1_role_id,
                channel_id: f1_channel_id,
            },
            debug: DebugConfig {
                channel_id: debug_channel_id,
            },
        })
    }
    
    pub fn get_main_channel_id(&self) -> Result<ChannelId, Box<dyn Error>> {
        ChannelId::from_str(&self.discord.channel_id)
            .map_err(|_| "Failed to parse discord.channel_id into a ChannelId".into())
    }
    
    pub fn get_f1_channel_id(&self) -> Result<ChannelId, Box<dyn Error>> {
        if self.f1.channel_id.is_empty() {
            self.get_main_channel_id()
        } else {
            ChannelId::from_str(&self.f1.channel_id)
                .map_err(|_| "Failed to parse f1.channel_id into a ChannelId".into())
        }
    }
    
    pub fn get_debug_channel_id(&self) -> Result<ChannelId, Box<dyn Error>> {
        if self.debug.channel_id.is_empty() {
            self.get_main_channel_id()
        } else {
            ChannelId::from_str(&self.debug.channel_id)
                .map_err(|_| "Failed to parse debug.channel_id into a ChannelId".into())
        }
    }
}

pub async fn setup() -> Result<(MySqlPool, String, ChannelId), Box<dyn Error>> {
    let config = BotConfig::new()?;

    // Establish connection to the database
    let db_pool = MySqlPoolOptions::new()
        .max_connections(10) // Adjust the maximum number of connections as needed
        .acquire_timeout(Duration::from_secs(10)) // Set a connection timeout
        .connect(&config.database.url)
        .await
        .map_err(|e| format!("Failed to connect to the database: {}", e))?;

    let channel_id = config.get_main_channel_id()?;

    Ok((db_pool, config.discord.token, channel_id))
}
