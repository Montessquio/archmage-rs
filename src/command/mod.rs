use eyre::Result;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::*;
use serenity::{
    model::application::interaction::application_command::ApplicationCommandInteraction, prelude::*,
};
use tracing::{event, Level};

use crate::archmage::Archmage;

mod misc;
mod music;
use self::misc::CreateApplicationCommandsMiscExt;
use self::music::CreateApplicationCommandsMusicExt;

impl Archmage {
    pub async fn register_commands_for_guild(&self, guild: &GuildId, ctx: &Context) -> Result<()> {
        let _ = GuildId::set_application_commands(guild, &ctx.http, |commands| {
            commands.register_misc_commands().register_music_commands()
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
        macro_rules! handler {
            ($fn:path) => {
                if $fn(start_time, command, ctx).await? {
                    return Ok(());
                }
            };
        }

        // Fast-returns Err if the handler errors
        // If the handler returns `true`, fast-exits
        // with `Ok(())`. If the handler returns
        // `false`, continues to the next handler.
        handler!(misc::misc_handler);
        handler!(music::music_handler);

        // No handler consumed the command!
        // Handle Unimplemented
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
