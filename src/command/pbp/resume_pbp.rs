use crate::{archmage::ArchmageError, command::{pbp::ChannelFlags, reply}, db::ArchmageDatabase};
use serenity::{
    client::Context,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
};

pub async fn run(
    ctx: &mut Context,
    command: &mut ApplicationCommandInteraction,
    db: &ArchmageDatabase,
) -> Result<(), ArchmageError> {
    let guild = match command.guild_id {
        Some(g) => g,
        None => return Err(ArchmageError::NoGuild),
    };

    let pbp = db.get_pbp(guild, command.channel_id).await?.map(|mut pbp| {
        pbp.flags.set(ChannelFlags::PAUSED, true);
        pbp
    });

    match pbp {
        None => {
            command
                .create_interaction_response(&ctx.http, |r| {
                    reply()
                        .title("None Found".to_string())
                        .desc("No play-by-post schedule found in this channel!".to_string())
                        .finish(r)
                })
                .await?;
        }
        Some(pbp) => {
            db.set_pbp(&pbp).await?;
            let timer_pretty = {
                let seconds = pbp.time_remaining.num_seconds() % 60;
                let minutes = (pbp.time_remaining.num_seconds() / 60) % 60;
                let hours = (pbp.time_remaining.num_seconds() / 60) / 60;
                format!(
                    "{}:{}:{}",
                    hours,
                    minutes,
                    seconds,
                )
            };
            command
                .create_interaction_response(&ctx.http, |r| {
                    reply()
                        .title("Timer Resumed!".to_string())
                        .desc(format!("Time Remaining: {timer_pretty}"))
                        .finish(r)
                })
                .await?;
        },
    };

    Ok(())
}