use crate::{archmage::ArchmageError, command::reply, db::ArchmageDatabase};
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

    let pbp = db.get_pbp(guild, command.channel_id).await?;

    let channel_name = match command.channel_id.name(&ctx.cache).await {
        Some(s) => s,
        None => "No Channel Name".to_string(),
    };

    match pbp {
        None => command
                .create_interaction_response(&ctx.http, |r| {
                    reply()
                        .title("None Found".to_string())
                        .desc("No play-by-post schedule found in this channel!".to_string())
                        .finish(r)
                })
                .await?,
        Some(pbp) => {
            let pbp_pretty = pbp.pretty_print()?;
            command
            .create_interaction_response(&ctx.http, |r| {
                reply()
                    .title(format!("Play-by-post schedule for #{}", channel_name))
                    .field("", &pbp_pretty, false)
                    .field("", &pbp.to_string(), false)
                    .finish(r)
            })
            .await?;
        }
    };

    Ok(())
}
