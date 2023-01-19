use serenity::builder::CreateApplicationCommand;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("leave")
    .description("Leave the current voice channel, stopping any music that may be playing and clearing the queue")
}