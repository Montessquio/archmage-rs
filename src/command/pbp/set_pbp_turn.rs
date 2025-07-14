use super::super::CommandOptionsRetrievalExt;
use crate::{archmage::ArchmageError, command::{reply, CommandOptionsInterpretationExt}, db::ArchmageDatabase};
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

    let new_turn_index = command
        .data
        .options
        .first()
        .find("pbp")
        .find("pbp-string")
        .interpret(CommandOptionType::Integer)
        .cloned();

    let new_turn_index = match new_turn_index.and_then(|i| i.as_i64()) {
        Some(i) => i,
        None => return Err(ArchmageError::BadParamType("Invalid or Missing new turn index!".to_owned())),
    };

    let pbp = db.get_pbp(guild, command.channel_id).await?.map(|mut pbp| {
        pbp.current_turn_index = new_turn_index.checked_rem(pbp.intervals.len().saturating_sub(1) as i64).unwrap_or(0);
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
        }
    };

    Ok(())
}
