use chrono::{Datelike, Local, NaiveDate, Utc};
use log::{error, info, warn};
use serenity::{
    all::{
        ChannelId, CommandInteraction, CreateInteractionResponseFollowup, UserId,
    },
    builder::{CreateEmbed, CreateEmbedFooter, CreateMessage},
    model::{Timestamp, mention::Mention, id::RoleId},
    prelude::*,
};
use sqlx::MySqlPool;
use std::str::FromStr;

use crate::commands::f1::api::{fetch_f1_calendar, find_next_race, is_thursday};
use crate::commands::f1::embed::create_race_embed;
use crate::commands::f1::tokens::{get_user_tokens, use_token, get_all_user_tokens, get_user_token_usage, get_all_token_usage}; // Import token functions
use crate::MySqlPoolKey; // Import MySqlPoolKey from main.rs
use crate::setup::BotConfig;

// List of admin user IDs (Discord user IDs as i64)
const ADMIN_USER_IDS: [i64; 1] = [
    98443943032684544, // Added as requested in task
];

// Function to check if a user is an admin
fn is_admin(user_id: i64) -> bool {
    ADMIN_USER_IDS.contains(&user_id)
}

// Command handler for the f1 command and its subcommands
pub async fn handle_commands(ctx: &Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Defer the response to buy time to fetch data
    command.defer(&ctx.http).await?;

    // Get user ID
    let user_id = command.user.id.get() as i64;

    // Get the database pool from context data
    let data = ctx.data.read().await;
    let pool_lock = data
        .get::<MySqlPoolKey>()
        .ok_or("Database pool not found in context")?;

    let pool = pool_lock.lock().await;

    // Check command type and only use token for specific token-consuming commands
    // Currently, no commands consume tokens, but this structure allows for future commands that might
    match command.data.options.get(0) {
        Some(option) => match option.name.as_str() {
            "next" => show_next_race(ctx, command).await?,
            "season" => show_season_races(ctx, command).await?,
            "tokens" => show_tokens(ctx, command, user_id, &pool).await?,
            "rules" => show_rules(ctx, command).await?,
            "swap" => record_team_swap(ctx, command, user_id, &pool).await?,
            "register" => register_user(ctx, command, user_id, &pool).await?,
            "history" => show_token_history(ctx, command, user_id, &pool).await?,
            _ => {
                command.create_followup(&ctx.http, CreateInteractionResponseFollowup::new()
                    .content("Unknown subcommand.")
                ).await?;
            }
        },
        None => {
            // Default to showing the next race if no subcommand specified
            show_next_race(ctx, command).await?
        }
    }

    async fn show_tokens(
        ctx: &Context, 
        command: &CommandInteraction,
        user_id: i64,
        pool: &MySqlPool
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Only admins can see all tokens
        if !is_admin(user_id) {
            // For non-admins, just show their own tokens
            match get_user_tokens(&pool, user_id).await {
                Ok(Some(tokens)) => {
                    let message = CreateInteractionResponseFollowup::new()
                        .content(format!("You have {} F1 fantasy team swap tokens remaining. Use the `/f1 swap` command when you change your team.", tokens));
                    command.create_followup(&ctx.http, message).await?;
                    return Ok(());
                },
                Ok(None) => {
                    let message = CreateInteractionResponseFollowup::new()
                        .content("You are not registered for F1 fantasy tokens. Ask an admin to add you.");
                    command.create_followup(&ctx.http, message).await?;
                    return Ok(());
                },
                Err(e) => {
                    error!("Failed to fetch user token: {:?}", e);
                    let message = CreateInteractionResponseFollowup::new()
                        .content("Failed to fetch your token information. Please try again later.");
                    command.create_followup(&ctx.http, message).await?;
                    return Ok(());
                }
            }
        }

        // For admins, show all users' tokens
        match get_all_user_tokens(&pool).await {
            Ok(tokens) => {
                let mut embed = CreateEmbed::default()
                    .title("F1 Fantasy Team Swap Tokens")
                    .description("List of users and their remaining team swap tokens:");

                for (user_id, tokens_remaining) in tokens {
                    // Try to fetch the username for each user ID
                    let username = match ctx.http.get_user(UserId::new(user_id as u64)).await {
                        Ok(user) => format!("{} ({})", user.name, user_id),
                        Err(_) => format!("Unknown User (ID: {})", user_id)
                    };
                    
                    embed = embed.field(
                        username,
                        format!("Team Swap Tokens: {}", tokens_remaining.unwrap_or(0)),
                        false,
                    );
                }

                let message = CreateInteractionResponseFollowup::new().add_embed(embed);
                command.create_followup(&ctx.http, message).await?;
            }
            Err(e) => {
                error!("Failed to fetch all user tokens: {:?}", e);
                let message = CreateInteractionResponseFollowup::new()
                    .content("Failed to fetch user tokens. Please try again later.");
                command.create_followup(&ctx.http, message).await?;
            }
        }
        Ok(())
    }
    
    // Function for users to record F1 fantasy team swaps (consumes a token)
    async fn record_team_swap(
        ctx: &Context,
        command: &CommandInteraction,
        user_id: i64,
        pool: &MySqlPool
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if user has tokens
        match get_user_tokens(&pool, user_id).await {
            Ok(Some(tokens)) => {
                if tokens <= 0 {
                    let message = CreateInteractionResponseFollowup::new()
                        .content("You have no F1 fantasy tokens remaining. Ask an admin for more tokens.");
                    command.create_followup(&ctx.http, message).await?;
                    return Ok(());
                }
                
                // User has tokens, try to consume one
                match use_token(&pool, user_id).await {
                    Ok(true) => {
                        // Token successfully consumed
                        info!("User {} used an F1 fantasy token. {} remaining.", user_id, tokens - 1);
                        
                        // Record that the user has swapped their team
                        let embed = CreateEmbed::default()
                            .title("F1 Fantasy Team Swap Recorded")
                            .description("Your team swap has been recorded! Each user gets 2 tokens per season to swap their team.")
                            .color(0xFF1801)
                            .field("Tokens Remaining", format!("{}", tokens - 1), false)
                            .footer(CreateEmbedFooter::new("Good luck with your new team selection!"))
                            .timestamp(Timestamp::now());
                            
                        let message = CreateInteractionResponseFollowup::new().add_embed(embed);
                        command.create_followup(&ctx.http, message).await?;
                    },
                    Ok(false) => {
                        warn!("User {} had tokens ({}) but use_token failed.", user_id, tokens);
                        let message = CreateInteractionResponseFollowup::new()
                            .content("An unexpected error occurred while using your token. Please try again.");
                        command.create_followup(&ctx.http, message).await?;
                    },
                    Err(e) => {
                        error!("Database error consuming token for user {}: {:?}", user_id, e);
                        let message = CreateInteractionResponseFollowup::new()
                            .content("A database error occurred while using your token.");
                        command.create_followup(&ctx.http, message).await?;
                    }
                }
            },
            Ok(None) => {
                // User not found in the token system
                let message = CreateInteractionResponseFollowup::new()
                    .content("You are not registered for F1 fantasy tokens. Ask an admin to add you.");
                command.create_followup(&ctx.http, message).await?;
            },
            Err(e) => {
                error!("Database error fetching tokens for user {}: {:?}", user_id, e);
                let message = CreateInteractionResponseFollowup::new()
                    .content("A database error occurred while checking your tokens.");
                command.create_followup(&ctx.http, message).await?;
            }
        }
        
        Ok(())
    }
    
    // Function for admins to register users for F1 fantasy tokens
    async fn register_user(
        ctx: &Context,
        command: &CommandInteraction,
        admin_id: i64,
        pool: &MySqlPool
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if the command user is an admin
        if !is_admin(admin_id) {
            let message = CreateInteractionResponseFollowup::new()
                .content("You must be an admin to register users for F1 fantasy tokens.");
            command.create_followup(&ctx.http, message).await?;
            return Ok(());
        }
        
        // Get the target user from the command options
        // First get the "register" subcommand options
        let register_options = match &command.data.options.get(0).unwrap().value {
            serenity::all::CommandDataOptionValue::SubCommand(options) => options,
            _ => return Err("Failed to get subcommand options".into()),
        };
        
        // Then find the "user" option within the subcommand
        let target_user = match register_options.iter()
            .find(|opt| opt.name == "user")
            .map(|opt| &opt.value) {
                Some(serenity::all::CommandDataOptionValue::User(user_id)) => *user_id,
                _ => return Err("User parameter not found or invalid".into()),
            };
        
        let target_user_id = target_user.get() as i64;
        
        // Check if the user already has tokens
        match get_user_tokens(&pool, target_user_id).await {
            Ok(Some(_)) => {
                let message = CreateInteractionResponseFollowup::new()
                    .content(format!("User <@{}> is already registered for F1 fantasy tokens.", target_user_id));
                command.create_followup(&ctx.http, message).await?;
            },
            Ok(None) => {
                // Register the user
                match crate::commands::f1::tokens::add_user_token(&pool, target_user_id).await {
                    Ok(()) => {
                        info!("Admin {} registered user {} for F1 fantasy tokens", admin_id, target_user_id);
                        let message = CreateInteractionResponseFollowup::new()
                            .content(format!("User <@{}> has been registered for F1 fantasy tokens with 2 tokens.", target_user_id));
                        command.create_followup(&ctx.http, message).await?;
                    },
                    Err(e) => {
                        error!("Database error registering user {}: {:?}", target_user_id, e);
                        let message = CreateInteractionResponseFollowup::new()
                            .content("A database error occurred while registering the user.");
                        command.create_followup(&ctx.http, message).await?;
                    }
                }
            },
            Err(e) => {
                error!("Database error checking user {}: {:?}", target_user_id, e);
                let message = CreateInteractionResponseFollowup::new()
                    .content("A database error occurred while checking if the user is already registered.");
                command.create_followup(&ctx.http, message).await?;
            }
        }
        
        Ok(())
    }
    
    // Function to show token usage history
    async fn show_token_history(
        ctx: &Context,
        command: &CommandInteraction,
        user_id: i64,
        pool: &MySqlPool
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if user is registered
        match get_user_tokens(&pool, user_id).await {
            Ok(Some(_)) => {
                // User exists in the system, proceed
            },
            Ok(None) => {
                // User not found in the token system
                let message = CreateInteractionResponseFollowup::new()
                    .content("You are not registered for F1 fantasy tokens. Ask an admin to add you.");
                command.create_followup(&ctx.http, message).await?;
                return Ok(());
            },
            Err(e) => {
                error!("Database error checking user {}: {:?}", user_id, e);
                let message = CreateInteractionResponseFollowup::new()
                    .content("A database error occurred while checking your registration status.");
                command.create_followup(&ctx.http, message).await?;
                return Ok(());
            }
        }
        
        // Prepare the embed
        let mut embed = CreateEmbed::default();
        
        if is_admin(user_id) {
            // Admin can see all token usage
            match get_all_token_usage(&pool).await {
                Ok(usage) => {
                    if usage.is_empty() {
                        let message = CreateInteractionResponseFollowup::new()
                            .content("No token usage history found.");
                        command.create_followup(&ctx.http, message).await?;
                        return Ok(());
                    }
                    
                    embed = embed
                        .title("F1 Fantasy Token Usage History")
                        .description("Complete history of token usage by all users:")
                        .color(0xFF1801);
                    
                    for (i, (user_id, timestamp, reason)) in usage.iter().enumerate().take(25) {
                        // Limit to 25 entries to avoid embed size limits
                        let username = match ctx.http.get_user(UserId::new(*user_id as u64)).await {
                            Ok(user) => user.name,
                            Err(_) => format!("Unknown User (ID: {})", user_id)
                        };
                        
                        embed = embed.field(
                            format!("{}. {} on {}", i+1, username, timestamp),
                            format!("Reason: {}", reason),
                            false
                        );
                    }
                    
                    if usage.len() > 25 {
                        embed = embed.footer(CreateEmbedFooter::new(
                            format!("Showing 25 of {} total entries", usage.len())
                        ));
                    }
                },
                Err(e) => {
                    error!("Failed to fetch token usage history: {:?}", e);
                    let message = CreateInteractionResponseFollowup::new()
                        .content("Failed to fetch token usage history. Please try again later.");
                    command.create_followup(&ctx.http, message).await?;
                    return Ok(());
                }
            }
        } else {
            // Regular user can only see their own usage
            match get_user_token_usage(&pool, user_id).await {
                Ok(usage) => {
                    if usage.is_empty() {
                        let message = CreateInteractionResponseFollowup::new()
                            .content("You haven't used any F1 fantasy tokens yet.");
                        command.create_followup(&ctx.http, message).await?;
                        return Ok(());
                    }
                    
                    embed = embed
                        .title("Your F1 Fantasy Token Usage History")
                        .description("Record of when you've used your tokens:")
                        .color(0xFF1801);
                    
                    for (i, (timestamp, reason)) in usage.iter().enumerate() {
                        embed = embed.field(
                            format!("{}. {}", i+1, timestamp),
                            format!("Reason: {}", reason),
                            false
                        );
                    }
                },
                Err(e) => {
                    error!("Failed to fetch user token usage: {:?}", e);
                    let message = CreateInteractionResponseFollowup::new()
                        .content("Failed to fetch your token usage history. Please try again later.");
                    command.create_followup(&ctx.http, message).await?;
                    return Ok(());
                }
            }
        }
        
        let message = CreateInteractionResponseFollowup::new().add_embed(embed);
        command.create_followup(&ctx.http, message).await?;
        
        Ok(())
    }
    
    // Function to show F1 Fantasy rules
    async fn show_rules(ctx: &Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rules = "# F1 Fantasy Rules\n\n\
                     These are the rules for our F1 Fantasy competition:\n\n\
                     1. Each participant gets 2 tokens per season\n\
                     2. Tokens are used to record when you've swapped your fantasy team\n\
                     3. You must use a token each time you change your team lineup\n\
                     4. Once you've used all your tokens, you cannot change your team again\n\
                     5. The participant with the most points at the end of the season wins\n\n\
                     To record a team swap, use the `/f1 swap` command\n\
                     Ask an admin if you have any questions!";
                     
        let embed = CreateEmbed::default()
            .title("🏎️ F1 Fantasy Rules 🏎️")
            .description(rules)
            .color(0xFF1801)
            .footer(CreateEmbedFooter::new("Ask an admin to be added to the F1 Fantasy competition"))
            .timestamp(Timestamp::now());
            
        let message = CreateInteractionResponseFollowup::new().add_embed(embed);
        command.create_followup(&ctx.http, message).await?;
        
        Ok(())
    }

    Ok(())
}

// Function to check for upcoming F1 races and announce them on Thursdays
pub async fn check_upcoming_race(ctx: Context, _channel_id: ChannelId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Only proceed if today is Thursday
    if !is_thursday() {
        return Ok(());
    }
    
    info!("check_upcoming_race: It's Thursday, checking for upcoming F1 races...");
    
    // Load bot config
    let config = match BotConfig::new() {
        Ok(config) => config,
        Err(err) => {
            let error_msg = format!("Failed to load config: {}", err);
            error!("{}", error_msg);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg)));
        }
    };
    
    // Get F1 channel ID
    let f1_channel_id = match config.get_f1_channel_id() {
        Ok(channel_id) => channel_id,
        Err(err) => {
            let error_msg = format!("Failed to get F1 channel ID: {}", err);
            error!("{}", error_msg);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg)));
        }
    };
    
    // Parse F1 role ID
    let role_id = match config.f1.role_id.parse::<u64>() {
        Ok(id) => id,
        Err(err) => {
            let error_msg = format!("Failed to parse F1 role ID: {}", err);
            error!("{}", error_msg);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, error_msg)));
        }
    };
        
    // Fetch F1 calendar
    match fetch_f1_calendar().await {
        Ok(calendar) => {
            if let Some(next_race) = find_next_race(&calendar.mr_data.race_table.races) {
                // Calculate if the race is this weekend (within the next 4 days)
                let race_date = NaiveDate::from_str(&next_race.date).unwrap_or_default();
                let today = Local::now().date_naive();
                let days_until = race_date.signed_duration_since(today).num_days();
                
                if days_until <= 4 {
                    // Race is this weekend, send announcement
                    let embed = create_race_embed(&next_race);
                    
                    let role_mention = format!("{}", Mention::from(RoleId::new(role_id)));
                    let message = CreateMessage::new()
                        .content(format!("{} **F1 RACE WEEKEND ALERT!**", role_mention))
                        .add_embed(embed);
                        
                    if let Err(why) = f1_channel_id.send_message(&ctx.http, message).await {
                        error!("Error sending F1 race announcement: {:?}", why);
                    } else {
                        info!("check_upcoming_race: F1 race announcement sent successfully!");
                    }
                } else {
                    info!("check_upcoming_race: Next F1 race is in {} days, not announcing yet.", days_until);
                }
            } else {
                info!("check_upcoming_race: No upcoming F1 races found.");
            }
        }
        Err(e) => {
            warn!("check_upcoming_race: Failed to fetch F1 calendar: {:?}", e);
        }
    }
    
    Ok(())
}

// Command handler for the next race subcommand
async fn show_next_race(ctx: &Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Fetch F1 calendar
    match fetch_f1_calendar().await {
        Ok(calendar) => {
            if let Some(next_race) = find_next_race(&calendar.mr_data.race_table.races) {
                info!("show_next_race: Next F1 race: {}", next_race.race_name);
                let embed = create_race_embed(&next_race);

                // Respond with the embed
                let message = CreateInteractionResponseFollowup::new().add_embed(embed);
                command.create_followup(&ctx.http, message).await?;
            } else {
                info!("show_next_race: No upcoming F1 races found for the current season.");
                let message = CreateInteractionResponseFollowup::new()
                    .content("No upcoming F1 races found for the current season.");
                command.create_followup(&ctx.http, message).await?;
            }
        }
        Err(e) => {
            error!("show_next_race: Failed to fetch F1 calendar: {:?}", e);
            let message = CreateInteractionResponseFollowup::new()
                .content("Failed to fetch F1 calendar. Please try again later.");
            command.create_followup(&ctx.http, message).await?;
        }
    }

    Ok(())
}

// Function to show all races for the current season
async fn show_season_races(ctx: &Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Fetch F1 calendar
    match fetch_f1_calendar().await {
        Ok(calendar) => {
            let today = Local::now().date_naive();
            let races = &calendar.mr_data.race_table.races;

            if races.is_empty() {
                info!("show_season_races: No F1 races found for the current season.");
                let message = CreateInteractionResponseFollowup::new()
                    .content("No F1 races found for the current season.");
                command.create_followup(&ctx.http, message).await?;
                return Ok(());
            }

            // Create an embed with all races
            let embed = CreateEmbed::default();
            let embed = embed
                .title(format!("🏎️ {} F1 Season Calendar 🏎️", Utc::now().year()))
                .color(0xFF1801)
                .thumbnail("https://www.formula1.com/etc/designs/fom-website/images/f1_logo.png")
                .description("Here are all the races for the current Formula 1 season:")
                .footer(CreateEmbedFooter::new("Data provided by Ergast F1 API"))
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

            info!("show_season_races: Sending embed to channel.");
            // Respond with the embed
            let message = CreateInteractionResponseFollowup::new().add_embed(embed);
            command.create_followup(&ctx.http, message).await?;
        }
        Err(e) => {
            error!("show_season_races: Failed to fetch F1 calendar: {:?}", e);
            let message = CreateInteractionResponseFollowup::new()
                .content("Failed to fetch F1 calendar. Please try again later.");
            command.create_followup(&ctx.http, message).await?;
        }
    }

    Ok(())
}
