use serenity::{
    all::{CommandOptionType, CreateCommand},
    builder::CreateCommandOption,
};

// Function to register the F1 command with subcommands
pub fn register() -> CreateCommand {
    let next_option = CreateCommandOption::new(CommandOptionType::SubCommand, "next", "Show the next upcoming F1 race");
    let season_option = CreateCommandOption::new(CommandOptionType::SubCommand, "season", "Show all races for the current F1 season");
    
    CreateCommand::new("f1")
        .description("Check F1 race information")
        .dm_permission(true)
        .add_option(next_option)
        .add_option(season_option)
} 