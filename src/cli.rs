use clap::Parser;
use serenity::model::Timestamp;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliCommands {
    #[clap(short, long)]
    ///Wether or not the bot is gonna scrape data.
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
}
