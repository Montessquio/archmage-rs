use serenity::builder::CreateApplicationCommand;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("shuffle").description("Toggle whether or not to randomly select the next song from the queue")
}