use chrono::{Datelike, Local, NaiveDate, Utc};
use log::{error, info};
use serenity::{
    all::{
        ChannelId, CommandInteraction, CreateInteractionResponseFollowup,
    },
    builder::{CreateEmbed, CreateEmbedFooter, CreateMessage},
    model::Timestamp,
    prelude::*,
};
use std::str::FromStr;

use crate::commands::f1::api::{fetch_f1_calendar, find_next_race, is_thursday};
use crate::commands::f1::embed::create_race_embed;

// Command handler for the f1 command and its subcommands
pub async fn handle_commands(ctx: Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Defer the response to buy time to fetch data
    command.defer(&ctx.http).await?;
    
    match command.data.options.get(0) {
        Some(option) => match option.name.as_str() {
            "next" => show_next_race(ctx, command).await?,
            "season" => show_season_races(ctx, command).await?,
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
    
    Ok(())
}

// Function to check for upcoming F1 races and announce them on Thursdays
pub async fn check_upcoming_race(ctx: Context, _channel_id: ChannelId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Only proceed if today is Thursday
    if !is_thursday() {
        return Ok(());
    }
    
    info!("check_upcoming_race: It's Thursday, checking for upcoming F1 races...");
    
    // Use the specific channel ID
    let announcement_channel = ChannelId::new(449885929629548544); // Formula 1 channel ID
    
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
                    
                    let message = CreateMessage::new()
                        .content("<@&formula1> **F1 RACE WEEKEND ALERT!**")
                        .add_embed(embed);
                    if let Err(why) = announcement_channel.send_message(&ctx.http, message).await {
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
            error!("check_upcoming_race: Failed to fetch F1 calendar: {:?}", e);
        }
    }
    
    Ok(())
}

// Command handler for the next race subcommand
async fn show_next_race(ctx: Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
async fn show_season_races(ctx: Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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