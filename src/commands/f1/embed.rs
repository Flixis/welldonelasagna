use chrono::{Local, NaiveDate};
use serenity::{
    all::{CreateEmbed},
    builder::{CreateEmbedFooter},
    model::Timestamp,
};
use std::str::FromStr;

use crate::commands::f1::models::Race;

// Function to create a rich embed for the race announcement
pub fn create_race_embed(race: &Race) -> CreateEmbed {
    let race_date = NaiveDate::from_str(&race.date).unwrap_or_default();
    let today = Local::now().date_naive();
    let days_until = race_date.signed_duration_since(today).num_days();
    
    let embed = CreateEmbed::default();
    embed
        .title(format!("🏎️ Upcoming F1 Race: {} 🏎️", race.race_name))
        .color(0xFF1801) // F1 red color
        .thumbnail("https://www.formula1.com/etc/designs/fom-website/images/f1_logo.png")
        .description(format!(
            "**Round {}** of the Formula 1 Championship is coming up!",
            race.round
        ))
        .field(
            "Circuit",
            format!("{}", race.circuit.circuit_name),
            true
        )
        .field(
            "Location",
            format!("{}, {}", race.circuit.location.locality, race.circuit.location.country),
            true
        )
        .field(
            "Date & Time",
            format!("{} {}", race.date, if !race.time.is_empty() { &race.time } else { "TBA" }),
            true
        )
        .field(
            "Countdown",
            format!("**{}** days until race day!", days_until),
            false
        )
        .footer(CreateEmbedFooter::new("Data provided by Ergast F1 API"))
        .timestamp(Timestamp::now())
} 