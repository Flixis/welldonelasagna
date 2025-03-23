use serenity::{
    all::{CommandInteraction, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    builder::CreateEmbed,
    prelude::*,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("version").description("Show bot version information")
}

pub async fn show_version(ctx: Context, command: &CommandInteraction) {
    let version = env!("CARGO_PKG_VERSION");
    let build_id = env!("BUILD_ID");

    let embed = CreateEmbed::new()
        .title("Bot Version Info")
        .field("Version", version, true)
        .field("Build ID", build_id, true);

    if let Err(why) = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().add_embed(embed)
            ),
        )
        .await
    {
        println!("Cannot respond to slash command: {why}");
    }
}
