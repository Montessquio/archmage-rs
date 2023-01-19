use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("repeat").description("Enable or disable repeating the current song or queue")
    .create_option(|option| {
        option
            .name("setting")
            .description("What should I repeat?")
            .kind(CommandOptionType::String)
            .add_string_choice("song", "song")
            .add_string_choice("queue", "queue")
            .required(true)
    })
}