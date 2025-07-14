use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::*;
use eyre::Result;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("play").description("Resume playback or append a song to the queue")
    .create_option(|option| {
        option
            .name("song")
            .description("A link to YouTube or SoundCloud, or a search term")
            .kind(CommandOptionType::String)
            .required(false)
    })
}

pub fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<()> {

    Ok(())
}