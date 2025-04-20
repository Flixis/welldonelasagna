use chrono::{Datelike, Local, NaiveDate, Utc};
use log::{error, info, warn};
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use serenity::{
    all::{ChannelId, Http}, 
    builder::{CreateEmbed, CreateEmbedFooter, CreateMessage}, 
    model::{prelude::*, id::RoleId, mention::Mention, Timestamp}, 
    prelude::*
};
use dotenv::dotenv; // For loading .env
use sqlx::mysql::MySqlPoolOptions; // For creating the pool
use std::env; // For reading DATABASE_URL

use crate::cli::{DebugCommands, F1Commands};
use crate::commands::f1;
use crate::commands::f1::tokens::add_user_token; // Import add_user_token
use crate::setup::BotConfig;

// Define a simple error type that explicitly implements Send + Sync
#[derive(Debug)]
struct SendSyncError {
    message: String
}

impl SendSyncError {
    fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into()
        }
    }
}

impl fmt::Display for SendSyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for SendSyncError {}

// Safe to send between threads
unsafe impl Send for SendSyncError {}
unsafe impl Sync for SendSyncError {}

pub async fn run_debug_mode(debug_command: Option<DebugCommands>) -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("Debug mode activated");
    
    // If a specific debug command was provided in CLI args, execute it
    if let Some(cmd) = debug_command {
        execute_debug_command(cmd).await?;
    } else {
        // Interactive debug mode
        run_interactive_debug().await?;
    }
    
    Ok(())
}

async fn execute_debug_command(command: DebugCommands) -> Result<(), Box<dyn StdError + Send + Sync>> {
    match command {
        DebugCommands::F1 { command } => execute_f1_command(command).await?,
    }
    
    Ok(())
}

async fn execute_f1_command(command: F1Commands) -> Result<(), Box<dyn StdError + Send + Sync>> {
    match command {
        F1Commands::NextRace => {
            info!("Executing debug F1 NextRace command");
            debug_f1_next_race().await?;
        },
        F1Commands::Season => {
            info!("Executing debug F1 Season command");
            debug_f1_season().await?;
        },
        F1Commands::CheckUpcoming => {
            info!("Executing debug F1 CheckUpcoming command");
            debug_f1_check_upcoming_race().await?;
        },
        F1Commands::AddUserToken { user_id } => {
            debug_f1_add_user_token(user_id).await?;
        },
    }
    
    Ok(())
}

// Debug command to add a user to the F1 fantasy token system
async fn debug_f1_add_user_token(user_id_str: String) -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("Executing debug F1 AddUserToken command for user_id: {}", user_id_str);

    // Parse user_id
    let user_id = match user_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            let error_msg = format!("Invalid user_id format: '{}'. Must be a number.", user_id_str);
            println!("{}", error_msg);
            return Err(Box::new(SendSyncError::new(error_msg)));
        }
    };

    // Load .env file
    dotenv().ok(); // Ignore error if .env doesn't exist, maybe DATABASE_URL is set otherwise

    // Get DATABASE_URL
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| Box::new(SendSyncError::new("DATABASE_URL must be set in .env or environment")) as Box<dyn StdError + Send + Sync>)?;

    // Create a database pool specifically for this debug command
    println!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(2) // Small pool for debug command
        .connect(&database_url)
        .await
        .map_err(|e| Box::new(SendSyncError::new(format!("Failed to connect to database: {}", e))) as Box<dyn StdError + Send + Sync>)?;
    println!("Connected.");

    // Call the add_user_token function
    match add_user_token(&pool, user_id).await {
        Ok(_) => {
            let success_msg = format!("Successfully added/verified user {} in the F1 fantasy token system.", user_id);
            info!("{}", success_msg);
            println!("{}", success_msg);
        }
        Err(e) => {
            let error_msg = format!("Failed to add user {} to token system: {}", user_id, e);
            error!("{}", error_msg);
            println!("{}", error_msg);
            pool.close().await; // Close pool on error too
            return Err(Box::new(SendSyncError::new(error_msg)));
        }
    }

    pool.close().await; // Close the pool when done

    Ok(())
}


async fn run_interactive_debug() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("Interactive debug mode started. Type 'help' for available commands or 'exit' to quit.");
    
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();
    
    loop {
        print!("debug> ");
        io::stdout().flush().await?;
        
        if let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.is_empty() { continue; }
            
            match parts[0].to_lowercase().as_str() {
                "exit" | "quit" => {
                    info!("Exiting debug mode");
                    break;
                },
                "help" => print_help(),
                "f1" => {
                    if parts.len() < 2 {
                        println!("Missing F1 subcommand. Available: next, season, check_upcoming, add_user_token <user_id>");
                    } else {
                        match parts[1].to_lowercase().as_str() {
                            "next" => debug_f1_next_race().await?,
                            "season" => debug_f1_season().await?,
                            "check_upcoming" => debug_f1_check_upcoming_race().await?,
                            "add_user_token" => {
                                if parts.len() < 3 {
                                    println!("Missing user_id argument for add_user_token.");
                                } else {
                                    // Pass the user_id string directly
                                    debug_f1_add_user_token(parts[2].to_string()).await?;
                                }
                            },
                            _ => println!("Unknown F1 subcommand: {}", parts[1]),
                        }
                    }
                },
                _ => println!("Unknown command: {}. Type 'help' for available commands.", parts[0]),
            }
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("Available debug commands:");
    println!("  help                - Show this help message");
    println!("  exit | quit         - Exit debug mode");
    println!("  f1 next             - Show the next F1 race information");
    println!("  f1 season           - Show the F1 season calendar");
    println!("  f1 check_upcoming   - Trigger the upcoming race check (simulates Thursday check)");
    println!("  f1 add_user_token <user_id> - Add a user to the F1 fantasy token system");
}

// Debug implementation of F1 related commands

// Helper function to create Discord client with debug channel
async fn create_debug_client() -> Result<(Client, ChannelId), Box<dyn StdError + Send + Sync>> {
    // Load config
    let config = BotConfig::new()
        .map_err(|e| Box::new(SendSyncError::new(format!("Failed to load config: {}", e))) as Box<dyn StdError + Send + Sync>)?;
        
    info!("Setting up Discord client for debug commands");
    
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
            
    let client = Client::builder(&config.discord.token, intents)
        .await
        .map_err(|e| Box::new(SendSyncError::new(e.to_string())) as Box<dyn StdError + Send + Sync>)?;
    
    // Use debug channel if configured, otherwise use main channel
    let channel_id = config.get_debug_channel_id()
        .map_err(|e| Box::new(SendSyncError::new(format!("Failed to get debug channel: {}", e))) as Box<dyn StdError + Send + Sync>)?;
    
    Ok((client, channel_id))
}

// Get F1 role ID from config
fn get_f1_role_id() -> Result<u64, Box<dyn StdError + Send + Sync>> {
    let config = BotConfig::new()
        .map_err(|e| Box::new(SendSyncError::new(format!("Failed to load config: {}", e))) as Box<dyn StdError + Send + Sync>)?;
    
    let role_id = config.f1.role_id.parse::<u64>()
        .map_err(|e| Box::new(SendSyncError::new(format!("Failed to parse F1 role ID: {}", e))) as Box<dyn StdError + Send + Sync>)?;
    
    Ok(role_id)
}

async fn debug_f1_next_race() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("Fetching next F1 race information (debug mode)");
    
    // Create a Discord client for sending messages to debug channel
    let (client, channel_id) = create_debug_client().await?;
    let http = client.http.clone();
    
    match f1::api::fetch_f1_calendar().await {
        Ok(calendar) => {
            if let Some(next_race) = f1::api::find_next_race(&calendar.mr_data.race_table.races) {
                // Create embed for Discord message
                let embed = f1::embed::create_race_embed(&next_race);
                
                // Print race information in the terminal
                println!("\n-----------------------------------");
                println!("🏎️ Next F1 Race: {}", next_race.race_name);
                println!("-----------------------------------");
                println!("Round: {}", next_race.round);
                println!("Circuit: {}", next_race.circuit.circuit_name);
                println!("Location: {}, {}", next_race.circuit.location.locality, next_race.circuit.location.country);
                println!("Date: {} {}", next_race.date, if !next_race.time.is_empty() { &next_race.time } else { "TBA" });
                println!("-----------------------------------");
                println!("Sending to Discord channel: {}", channel_id);
                println!("-----------------------------------\n");
                
                // Send to Discord
                let message = CreateMessage::new().add_embed(embed);
                match channel_id.send_message(&http, message).await {
                    Ok(_) => {
                        info!("Next F1 race information sent to Discord successfully");
                        println!("Successfully sent to Discord!");
                    },
                    Err(e) => {
                        warn!("Failed to send F1 race info to Discord: {:?}", e);
                        println!("Failed to send to Discord: {}", e);
                        return Err(Box::new(SendSyncError::new(e.to_string())));
                    }
                }
            } else {
                println!("No upcoming F1 races found for the current season.");
            }
        },
        Err(e) => {
            warn!("Failed to fetch F1 calendar in debug mode: {:?}", e);
            println!("Failed to fetch F1 calendar: {}", e);
            return Err(Box::new(SendSyncError::new(e.to_string())));
        }
    }
    
    Ok(())
}

async fn debug_f1_season() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("Fetching F1 season calendar (debug mode)");
    
    // Create a Discord client for sending messages to debug channel
    let (client, channel_id) = create_debug_client().await?;
    let http = client.http.clone();
    
    match f1::api::fetch_f1_calendar().await {
        Ok(calendar) => {
            let races = &calendar.mr_data.race_table.races;
            
            if races.is_empty() {
                println!("No F1 races found for the current season.");
                return Ok(());
            }
            
            // Display in terminal
            println!("\n====================================");
            println!("🏎️ F1 Season Calendar 🏎️");
            println!("====================================");
            
            for race in races {
                println!("\nRound {}:", race.round);
                println!("  Race: {}", race.race_name);
                println!("  Circuit: {}", race.circuit.circuit_name);
                println!("  Location: {}, {}", race.circuit.location.locality, race.circuit.location.country);
                println!("  Date: {} {}", race.date, if !race.time.is_empty() { &race.time } else { "TBA" });
            }
            
            println!("\n====================================");
            println!("Sending to Discord channel: {}", channel_id);
            println!("====================================\n");
            
            // Create F1 season embed directly
            let today = Local::now().date_naive();
            
            let embed = CreateEmbed::default()
                .title(format!("🏎️ {} F1 Season Calendar 🏎️", Utc::now().year()))
                .color(0xFF1801)
                .thumbnail("https://www.formula1.com/etc/designs/fom-website/images/f1_logo.png")
                .description("Here are all the races for the current Formula 1 season:")
                .footer(CreateEmbedFooter::new("Data provided by Ergast F1 API - Debug Mode"))
                .timestamp(Timestamp::now());
            
            // Add fields for each race with status
            let embed = races.iter().fold(embed, |embed, race| {
                let race_date = NaiveDate::from_str(&race.date).unwrap_or_default();
                let status = if race_date < today {
                    "✅ Completed"
                } else if race_date == today {
                    "🏁 Today!"
                } else {
                    "⏳ Upcoming"
                };
                
                let days_until = race_date.signed_duration_since(today).num_days();
                let time_info = if days_until < 0 {
                    format!("{} days ago", days_until.abs())
                } else if days_until == 0 {
                    "Today!".to_string()
                } else {
                    format!("In {} days", days_until)
                };
                
                embed.field(
                    format!("Round {} - {}", race.round, race.race_name),
                    format!(
                        "**Circuit:** {}\n**Location:** {}, {}\n**Date:** {} {}\n**Status:** {} ({})",
                        race.circuit.circuit_name,
                        race.circuit.location.locality,
                        race.circuit.location.country,
                        race.date,
                        if !race.time.is_empty() { &race.time } else { "TBA" },
                        status,
                        time_info
                    ),
                    false
                )
            });
            
            // Send to Discord
            let message = CreateMessage::new()
                .content("**F1 Season Calendar (Debug Mode)**")
                .add_embed(embed);
                
            match channel_id.send_message(&http, message).await {
                Ok(_) => {
                    info!("F1 season calendar sent to Discord successfully");
                    println!("Successfully sent to Discord!");
                },
                Err(e) => {
                    warn!("Failed to send F1 season calendar to Discord: {:?}", e);
                    println!("Failed to send to Discord: {}", e);
                    return Err(Box::new(SendSyncError::new(e.to_string())));
                }
            }
        },
        Err(e) => {
            warn!("Failed to fetch F1 calendar in debug mode: {:?}", e);
            println!("Failed to fetch F1 calendar: {}", e);
            return Err(Box::new(SendSyncError::new(e.to_string())));
        }
    }
    
    Ok(())
}

// Helper function to mimic the F1 race check but force it regardless of day
async fn force_f1_race_check(http: &Http, channel_id: ChannelId) -> Result<(), Box<dyn StdError + Send + Sync>> {
    // Fetch the F1 calendar
    match f1::api::fetch_f1_calendar().await {
        Ok(calendar) => {
            if let Some(next_race) = f1::api::find_next_race(&calendar.mr_data.race_table.races) {
                // In debug mode, we'll always announce the race
                let embed = f1::embed::create_race_embed(&next_race);
                
                // Create the role mention with role ID from config
                let role_id = get_f1_role_id()?;
                let role_mention = format!("{}", Mention::from(RoleId::new(role_id)));
                
                // Create the message with embed
                let message = CreateMessage::new()
                    .content(format!("{} **F1 RACE WEEKEND ALERT! (Debug Mode)**", role_mention))
                    .add_embed(embed);
                
                // Try to send the message
                match channel_id.send_message(http, message).await {
                    Ok(_) => {
                        info!("Debug mode: F1 race announcement sent successfully!");
                        println!("Successfully sent race announcement to Discord!");
                    },
                    Err(e) => {
                        error!("Debug mode: Error sending F1 race announcement: {:?}", e);
                        return Err(Box::new(SendSyncError::new(e.to_string())));
                    }
                }
            } else {
                println!("No upcoming F1 races found.");
            }
        },
        Err(e) => {
            error!("Debug mode: Failed to fetch F1 calendar: {:?}", e);
            return Err(Box::new(SendSyncError::new(e.to_string())));
        }
    }
    
    Ok(())
}

async fn debug_f1_check_upcoming_race() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("Triggering F1 race check on Discord (debug mode)");
    
    // Create a Discord client for sending messages to debug channel
    let (client, channel_id) = create_debug_client().await?;
    let http = client.http.clone();
    
    // Print status in terminal
    println!("\n-----------------------------------");
    println!("🏎️ F1 RACE WEEKEND ALERT!");
    println!("-----------------------------------");
    println!("Triggering the F1 race check on Discord...");
    println!("This will post to Discord channel: {}", channel_id);
    println!("-----------------------------------\n");
    
    // Force the race check directly with the HTTP client
    if let Err(e) = force_f1_race_check(&http, channel_id).await {
        warn!("Error checking F1 races: {:?}", e);
        println!("Failed to send to Discord: {}", e);
        return Err(e);
    }
    
    info!("F1 race check triggered successfully");
    println!("Successfully triggered F1 race check on Discord!");
    
    Ok(())
}
