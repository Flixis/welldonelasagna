use clap::{Parser, Subcommand};
use serenity::model::Timestamp;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliCommands {
    #[clap(short, long)]
    ///Whether or not the bot is gonna scrape data.
    pub scraping: bool,

    #[clap(short, long)]
    ///The amount of messages required before the bot tries to roll and qoute someone.
    pub roll_amount: Option<usize>,

    #[clap(long, requires("scraping"))]
    ///Starting scrape from date. Date format like <2028-01-01T00:00:00Z>.
    pub start_date: Option<Timestamp>,

    #[clap(long, requires("scraping"))]
    ///Ending scrape at date. Date format like <2028-01-01T00:00:00Z>.
    pub end_date: Option<Timestamp>,

    #[clap(short, long)]
    ///Enable debug mode to trigger events from the terminal without starting the bot.
    pub debug: bool,

    #[clap(subcommand)]
    ///Debug commands to trigger specific events.
    pub debug_command: Option<DebugCommands>,
}

#[derive(Subcommand, Clone)]
pub enum DebugCommands {
    /// F1 related commands
    F1 {
        #[clap(subcommand)]
        command: F1Commands,
    },
}

#[derive(Subcommand, Clone)]
pub enum F1Commands {
    /// Show the next race information
    NextRace,
    /// Show the full season calendar
    Season,
    /// Trigger the upcoming race check (simulates Thursday check)
    CheckUpcoming,
}
