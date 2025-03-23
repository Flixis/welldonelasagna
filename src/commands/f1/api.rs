use chrono::{Datelike, Local, NaiveDate, Utc, Weekday};
use log::info;
use std::str::FromStr;

use crate::commands::f1::models::*;

// Function to fetch F1 calendar for the current year
pub async fn fetch_f1_calendar() -> Result<F1Calendar, reqwest::Error> {
    let current_year = Utc::now().year().to_string();
    let url = format!("https://api.jolpi.ca/ergast/f1/{}/races.json", current_year);
    info!("Fetching F1 calendar for year: {}", current_year);
    
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    response.json::<F1Calendar>().await
}

// Function to find the next upcoming race
pub fn find_next_race(races: &[Race]) -> Option<Race> {
    let today = Local::now().date_naive();
    
    races.iter()
        .find(|race| {
            NaiveDate::from_str(&race.date).ok()
                .filter(|race_date| *race_date >= today)
                .is_some()
        })
        .cloned()
}

// Check if today is Thursday
pub fn is_thursday() -> bool {
    Local::now().weekday() == Weekday::Thu
} 