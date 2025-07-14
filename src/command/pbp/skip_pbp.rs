use crate::{archmage::ArchmageError, command::reply, db::ArchmageDatabase};
use serenity::{
    client::Context,
    json::Value,
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

    let new_turn_index = match command.data.options.first() {
        Some(cdo) => match cdo.kind {
            CommandOptionType::SubCommand => match cdo.kind {
                CommandOptionType::Integer => match &cdo.value {
                    Some(Value::Number(n)) => match n.as_i64() {
                        Some(n) => n,
                        None => return Err(ArchmageError::BadParamType("/subcommand/integer/number/i64".to_owned())),
                    },
                    _ => return Err(ArchmageError::BadParamType("/subcommand/integer/number".to_owned())),
                },
                _ => return Err(ArchmageError::BadParamType("/subcommand/integer".to_owned())),
            },
            _ => return Err(ArchmageError::BadParamType("/subcommand".to_owned())),
        },
        None => return Err(ArchmageError::NotEnoughParams),
    };

    let pbp = db.get_pbp(guild, command.channel_id).await?.map(|mut pbp| {
        pbp.current_turn_index = new_turn_index;
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
        },
    };

    Ok(())
}