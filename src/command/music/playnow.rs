use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("playnow").description("Insert a song at the top of the queue and skip to it")
    .create_option(|option| {
        option
            .name("song")
            .description("A link to YouTube or SoundCloud, or a search term")
            .kind(CommandOptionType::String)
            .required(true)
    })
}