use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("skip").description("Skip one or more songs in the queue")
    .create_option(|option| {
        option
            .name("amount")
            .description("How many songs to skip")
            .kind(CommandOptionType::Number)
            .required(false)
    })
}