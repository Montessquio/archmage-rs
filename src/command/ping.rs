use eyre::{eyre, Result};
use serenity::{
    all::{Color, CommandInteraction, CreateCommand, CreateEmbed, CreateInteractionResponseMessage, Timestamp},
    prelude::*,
};
use crate::{archmage::Archmage, command::handle_fn};
use super::ArchmageCommand;

pub struct PingCommand;

impl ArchmageCommand for PingCommand {
    fn register() -> Vec<(String, CreateCommand, super::HandleFn)> {
        vec![(
            "ping".to_owned(),
            CreateCommand::new("ping").description("Determine server command (not network) latency"),
            handle_fn!(Self::run),
        )]
    }
}

impl PingCommand {
    async fn run(
        _client: &Archmage,
        command: &CommandInteraction,
        ctx: &Context,
    ) -> Result<()> {
        command
            .create_response(
                &ctx.http,
                serenity::all::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().embed(
                        CreateEmbed::new()
                            .color(Color::from_rgb(0x00, 0xFF, 0x00))
                            .description(format!(
                                "Received {} UTC",
                                chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S%.6f")
                            ))
                            .title("Pong!")
                            .timestamp(Timestamp::now()),
                    ),
                ),
            )
            .await
            .map(|_| ())
            .map_err(|e| eyre!(e))
    }
}