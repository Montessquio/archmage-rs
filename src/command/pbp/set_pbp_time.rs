use crate::{archmage::ArchmageError, command::{reply, CommandOptionsInterpretationExt, CommandOptionsRetrievalExt}, db::ArchmageDatabase};
use chrono::Duration;
use serenity::{
    client::Context,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction,
    },
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

    let new_time_secs = command
        .data
        .options
        .first()
        .find("set-time")
        .find("seconds")
        .interpret(CommandOptionType::Integer)
        .cloned();

    let new_time_secs = match new_time_secs.and_then(|i| i.as_i64()) {
        Some(i) => i,
        None => return Err(ArchmageError::BadParamType("Invalid or Missing time!".to_owned())),
    };

    let pbp = db.get_pbp(guild, command.channel_id).await?.map(|mut pbp| {
        pbp.time_remaining = Duration::seconds(new_time_secs);
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
                .await?
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
                        .desc(format!("Timer set for {timer_pretty}"))
                        .finish(r)
                })
                .await?;
        }
    };

    Ok(())
}
