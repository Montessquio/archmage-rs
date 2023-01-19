use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;

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