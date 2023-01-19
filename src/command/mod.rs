use eyre::Result;
use serenity::model::prelude::*;
use serenity::{
    model::application::interaction::application_command::ApplicationCommandInteraction, prelude::*,
};
use serenity::model::application::interaction::InteractionResponseType;
use tracing::{event, Level};

use crate::archmage::Archmage;

mod music;
mod ping;
mod roll;
use self::music::CreateApplicationCommandsMusicExt;

impl Archmage {
    pub async fn register_commands_for_guild(&self, guild: &GuildId, ctx: &Context) -> Result<()> {
        let _ = GuildId::set_application_commands(guild, &ctx.http, |commands| {
            commands
                .create_application_command(|command| ping::register(command))
                .create_application_command(|command| roll::register(command))
                .create_application_command(|command| roll::register_short(command))
                .register_music_commands()
        })
        .await?;

        Ok(())
    }

    pub async fn handle_command(
        &self,
        start_time: chrono::NaiveDateTime,
        command: &ApplicationCommandInteraction,
        ctx: &Context,
    ) -> Result<()> {
        match command.data.name.as_str() {
            "ping" => ping::run(start_time, command, &ctx).await,
            "roll" | "r" => roll::run(command, &ctx).await,
            _ => self.handle_unimplemented(command, &ctx).await,
        }
    }

    async fn handle_unimplemented(
        &self,
        command: &ApplicationCommandInteraction,
        ctx: &Context,
    ) -> Result<()> {
        let response = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| {
                            m.embed(|e| {
                                use serenity::utils::Color;
                                e.color(Color::from_rgb(0x00, 0xFF, 0x00))
                                 .description("Archmage is still working on this spell! Please try again later.")
                                 .title("Not yet implemented!")
                                 .timestamp(chrono::Utc::now().to_rfc3339())
                            })
                        })
                    }
                ).await;

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
