use serenity::builder::CreateApplicationCommands;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

use eyre::Result;

mod play;
mod playnow;
mod pause;
mod stop;
mod leave;
mod repeat;
mod skip;
mod queue;
mod shuffle;
mod nowplaying;

pub trait CreateApplicationCommandsMusicExt {
    fn register_music_commands(&mut self) -> &mut Self;
}

impl CreateApplicationCommandsMusicExt for CreateApplicationCommands {
    fn register_music_commands(&mut self) -> &mut Self {
        self
        .create_application_command(|command| play::register(command))
        .create_application_command(|command| playnow::register(command))
        .create_application_command(|command| pause::register(command))
        .create_application_command(|command| stop::register(command))
        .create_application_command(|command| leave::register(command))
        .create_application_command(|command| repeat::register(command))
        .create_application_command(|command| skip::register(command))
        .create_application_command(|command| queue::register(command))
        .create_application_command(|command| shuffle::register(command))
        .create_application_command(|command| nowplaying::register(command))
    }
}

pub async fn music_handler(_start_time: chrono::NaiveDateTime, command: &ApplicationCommandInteraction, _ctx: &Context) -> Result<bool> {
    match command.data.name.as_str() {
        "stub" => Ok(()),
        _ => return Ok(false),
    }.map(|_| true)
}
