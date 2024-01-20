use clap::Parser;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliCommands {
    #[clap(short, long)]
    ///Wether or not the bot is gonna scrape data.
    pub scraping: Option<bool>,

    #[clap(short, long)]
    ///The amount of messages required before the bot tries to roll and qoute someone.
    pub roll_amount: Option<usize>,
}
