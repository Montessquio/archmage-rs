use eyre::{bail, eyre, Result};
use serenity::{
    all::{Color, CommandDataOptionValue, CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponseMessage, Timestamp},
    prelude::*,
};

pub async fn run(
    command: &CommandInteraction,
    ctx: &Context,
) -> Result<()> {
    let url = {
        let option = &command
        .data
        .options
        .first()
        .ok_or(eyre!("Expected a valid video URL"))?
        .value;

        match option {
            CommandDataOptionValue::String(input) => input,
            _ => bail!("Unexpected input type"),
        }
    };

    

    command
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(
                    CreateEmbed::new()
                        .color(Color::from_rgb(0x00, 0xFF, 0x00))
                        .description(format!(
                            "Received {} UTC",
                            Timestamp::now().to_rfc3339().unwrap()
                        ))
                        .title(url)
                        .timestamp(Timestamp::now()),
                ),
            ),
        )
        .await
        .map(|_| ())
        .map_err(|e| eyre!(e))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play")
    .description("Play audio from a youtube link in your current voice channel.")
    .add_option(
        CreateCommandOption::new(
            CommandOptionType::String,
            "link",
            "A link to the youtube video to play.",
        )
        .required(true),
    )
}
