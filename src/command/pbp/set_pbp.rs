use crate::{
    archmage::ArchmageError,
    command::{pbp::PbpSchedule, reply, CommandOptionsInterpretationExt, CommandOptionsRetrievalExt},
    db::ArchmageDatabase,
};
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

    let raw_str = command
        .data
        .find("set")
        .find("pbp-string")
        .interpret(CommandOptionType::String)
        .cloned();

    let raw_str = match raw_str.and_then(|s| s.as_str().map(|s| s.to_owned())) {
        Some(s) => s,
        None => return Err(ArchmageError::BadParamType("Invalid or Missing schedule string!".to_owned())),
    };
    
    let pbp = PbpSchedule::parse_str(guild.0, command.channel_id.0, raw_str)?;

    let old_pbp = db.set_pbp(&pbp).await?;

    let channel_name = match command.channel_id.name(&ctx.cache).await {
        Some(s) => s,
        None => "No Channel Name".to_string(),
    };

    let old_pbp = match old_pbp {
        None => String::new(),
        Some(old_pbp) => format!("\n\n{old_pbp}"),
    };

    let pbp_pretty = pbp.pretty_print()?;
    command
        .create_interaction_response(&ctx.http, |r| {
            reply()
                .title(format!("Set play-by-post schedule for #{channel_name}"))
                .desc(format!("{pbp_pretty}\n\n{pbp}{old_pbp}"))
                .finish(r)
        })
        .await?;

    Ok(())
}
