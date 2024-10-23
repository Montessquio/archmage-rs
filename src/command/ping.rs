use eyre::{eyre, Result};
use serenity::{
    all::{Color, CommandInteraction, CreateCommand, CreateEmbed, CreateInteractionResponseMessage, Timestamp},
    prelude::*,
};

pub async fn run(
    time_start: chrono::NaiveDateTime,
    command: &CommandInteraction,
    ctx: &Context,
) -> Result<()> {
    let time_end = chrono::Utc::now().naive_utc();

    command
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(
                    CreateEmbed::new()
                        .color(Color::from_rgb(0x00, 0xFF, 0x00))
                        .description(format!(
                            "Received {} UTC",
                            time_start.format("%Y-%m-%d %H:%M:%S%.6f")
                        ))
                        .title(format!(
                            "Pong! ({}ms)",
                            (time_end.time() - time_start.time()).num_milliseconds()
                        ))
                        .timestamp(Timestamp::now()),
                ),
            ),
        )
        .await
        .map(|_| ())
        .map_err(|e| eyre!(e))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("Determine server command (not network) latency")
}
