use eyre::Result;
use serenity::{
    all::{Context, CreateEmbed, CreateInteractionResponseMessage},
    model::prelude::*,
};
use tracing::{event, Level};

use crate::archmage::Archmage;

mod music;
mod ping;
mod roll;

impl Archmage {
    pub async fn register_commands_for_guild(&self, guild: &GuildId, ctx: &Context) -> Result<()> {
        let mut commands = vec![ping::register(), roll::register(), roll::register_short()];
        commands.append(&mut music::register_all());

        let _commands = guild
            .set_commands(
                &ctx.http,
                commands,
            )
            .await?;

        Ok(())
    }

    pub async fn handle_command(
        &self,
        start_time: chrono::NaiveDateTime,
        command: &CommandInteraction,
        ctx: &Context,
    ) -> Result<()> {
        match command.data.name.as_str() {
            "ping" => ping::run(start_time, command, ctx).await,
            "roll" | "r" => roll::run(command, ctx).await,
            "play" => music::play::run(command, ctx).await,
            _ => self.handle_unimplemented(command, ctx).await,
        }
    }

    async fn handle_unimplemented(
        &self,
        command: &CommandInteraction,
        ctx: &Context,
    ) -> Result<()> {
        let response = command
            .create_response(
                &ctx.http,
                serenity::all::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().embed(
                        CreateEmbed::new()
                            .color(Color::from_rgb(0x00, 0xFF, 0x00))
                            .description(
                                "Archmage is still working on this spell! Please try again later.",
                            )
                            .title("Not yet implemented!")
                            .timestamp(Timestamp::now()),
                    ),
                ),
            )
            .await;

        if let Err(e) = response {
            event!(
                Level::ERROR,
                error = &format!("{}", e).as_str(),
                "DOUBLE FAULT! Error sending error message to user channel"
            )
        }

        Ok(())
    }
}
