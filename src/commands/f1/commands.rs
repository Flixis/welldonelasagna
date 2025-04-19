use chrono::{Datelike, Local, NaiveDate, Utc};
use log::{error, info, warn};
use serenity::{
    all::{
        ChannelId, CommandInteraction, CreateInteractionResponseFollowup,
    },
    builder::{CreateEmbed, CreateEmbedFooter, CreateMessage},
    model::{Timestamp, mention::Mention, id::RoleId},
    prelude::*,
};
use std::str::FromStr;

use crate::commands::f1::api::{fetch_f1_calendar, find_next_race, is_thursday};
use crate::commands::f1::embed::create_race_embed;
use crate::commands::f1::tokens::{get_user_tokens, use_token, get_all_user_tokens}; // Import token functions
use crate::MySqlPoolKey; // Import MySqlPoolKey from main.rs
use crate::setup::BotConfig;

// Command handler for the f1 command and its subcommands
pub async fn handle_commands(ctx: &Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Defer the response to buy time to fetch data
    command.defer(&ctx.http).await?;

    // --- F1 Fantasy Token Check ---
    let user_id = command.user.id.get() as i64; // Get user ID as i64

    // Get the database pool from context data
    let data = ctx.data.read().await;
    let pool_lock = data
        .get::<MySqlPoolKey>()
        .ok_or("Database pool not found in context")?; // Handle potential missing pool

    let pool = pool_lock.lock().await;

    // Check user's token status
    match get_user_tokens(&pool, user_id).await {
        Ok(Some(tokens_remaining)) => {
            if tokens_remaining > 0 {
                // User has tokens, try to consume one
                match use_token(&pool, user_id).await {
                    Ok(true) => {
                        info!("User {} used an F1 fantasy token. {} remaining.", user_id, tokens_remaining - 1);
                        // Token consumed, proceed with the command
                    }
                    Ok(false) => {
                        // This case should ideally not happen if get_user_tokens returned > 0, but handle defensively
                        warn!("User {} had tokens ({}) but use_token failed.", user_id, tokens_remaining);
                        let message = CreateInteractionResponseFollowup::new()
                            .content("An unexpected error occurred while using your token. Please try again.");
                        command.create_followup(&ctx.http, message).await?;
                        return Ok(()); // Stop processing
                    }
                    Err(e) => {
                        error!("Database error consuming token for user {}: {:?}", user_id, e);
                        let message = CreateInteractionResponseFollowup::new()
                            .content("A database error occurred while using your token.");
                        command.create_followup(&ctx.http, message).await?;
                        return Ok(()); // Stop processing
                    }
                }
            } else {
                // User has no tokens left
                info!("User {} attempted to use F1 command but has no tokens left.", user_id);
                let message = CreateInteractionResponseFollowup::new()
                    .content("You have no F1 fantasy tokens remaining for this season!");
                command.create_followup(&ctx.http, message).await?;
                return Ok(()); // Stop processing
            }
        }
        Ok(None) => {
            // User not found in the token system
            info!("User {} attempted to use F1 command but is not registered for fantasy tokens.", user_id);
            let message = CreateInteractionResponseFollowup::new()
                .content("You are not registered for F1 fantasy tokens. Ask an admin to add you.");
            command.create_followup(&ctx.http, message).await?;
            return Ok(()); // Stop processing
        }
        Err(e) => {
            // Database error fetching tokens
            error!("Database error fetching tokens for user {}: {:?}", user_id, e);
            let message = CreateInteractionResponseFollowup::new()
                .content("A database error occurred while checking your F1 fantasy tokens.");
            command.create_followup(&ctx.http, message).await?;
            return Ok(()); // Stop processing
        }
    }
    // --- End F1 Fantasy Token Check ---

    // Proceed with original command logic if token check passed
    match command.data.options.get(0) {
        Some(option) => match option.name.as_str() {
            "next" => show_next_race(ctx, command).await?,
            "season" => show_season_races(ctx, command).await?,
            "tokens" => show_tokens(ctx, command).await?,
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

    async fn show_tokens(ctx: &Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data.read().await;
        let pool_lock = data
            .get::<MySqlPoolKey>()
            .ok_or("Database pool not found in context")?; // Handle potential missing pool

        let pool = pool_lock.lock().await;

        match get_all_user_tokens(&pool).await {
            Ok(tokens) => {
                let mut embed = CreateEmbed::default()
                    .title("F1 Fantasy Tokens")
                    .description("List of users and their remaining tokens:");

                for (user_id, tokens_remaining) in tokens {
                    embed = embed.field(
                        format!("User ID: {}", user_id),
                        format!("Tokens Remaining: {}", tokens_remaining.unwrap_or(0)),
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
