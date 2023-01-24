use serenity::{builder::CreateApplicationCommand, prelude::*};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use eyre::Result;

pub async fn run(time_start: chrono::NaiveDateTime, command: &ApplicationCommandInteraction, ctx: &Context) -> Result<()> {
    let time_end = chrono::Utc::now().naive_utc();

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| {
                    m.embed(|e| {
                        use serenity::utils::Color;
                        e.color(Color::from_rgb(0x00, 0xFF, 0x00))
                         .description(format!(
                            "Received {} UTC",
                            time_start.format("%Y-%m-%d %H:%M:%S%.6f")
                         ))
                         .title(format!("Pong! ({}ms)", (time_end.time() - time_start.time()).num_milliseconds()))
                         .timestamp(chrono::Utc::now().to_rfc3339())
                    })
                })
            }
        ).await?;
    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("Determine server command (not network) latency")
}