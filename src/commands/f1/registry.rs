use serenity::{
    all::{CommandOptionType, CreateCommand},
    builder::CreateCommandOption,
};

// Function to register the F1 command with subcommands
pub fn register() -> CreateCommand {
    let next_option = CreateCommandOption::new(CommandOptionType::SubCommand, "next", "Show the next upcoming F1 race");
    let season_option = CreateCommandOption::new(CommandOptionType::SubCommand, "season", "Show all races for the current F1 season");
    let tokens_option = CreateCommandOption::new(CommandOptionType::SubCommand, "tokens", "Check your F1 fantasy tokens");
    let rules_option = CreateCommandOption::new(CommandOptionType::SubCommand, "rules", "Show the F1 Fantasy rules");
    let swap_option = CreateCommandOption::new(CommandOptionType::SubCommand, "swap", "Record that you've swapped your F1 fantasy team (uses one token)");
    
    // Add register command for admins to add users to the F1 fantasy system
    let register_option = CreateCommandOption::new(CommandOptionType::SubCommand, "register", "Register a user for F1 fantasy tokens (admin only)")
        .add_sub_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "The user to register")
                .required(true)
        );
    
    // Add history command to view token usage history
    let history_option = CreateCommandOption::new(CommandOptionType::SubCommand, "history", "View your F1 fantasy token usage history");
    
    CreateCommand::new("f1")
        .description("Check F1 race information")
        .dm_permission(true)
        .add_option(next_option)
        .add_option(season_option)
        .add_option(tokens_option)
        .add_option(rules_option)
        .add_option(swap_option)
        .add_option(register_option)
        .add_option(history_option)
}
