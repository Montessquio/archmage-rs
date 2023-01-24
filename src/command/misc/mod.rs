use serenity::builder::CreateApplicationCommands;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

use eyre::Result;

mod ping;
mod roll;

pub trait CreateApplicationCommandsMiscExt {
    fn register_misc_commands(&mut self) -> &mut Self;
}

impl CreateApplicationCommandsMiscExt for CreateApplicationCommands {
    fn register_misc_commands(&mut self) -> &mut Self {
        self
        .create_application_command(|command| ping::register(command))
        .create_application_command(|command| roll::register(command))
        .create_application_command(|command| roll::register_short(command))
    }
}

pub async fn misc_handler(start_time: chrono::NaiveDateTime, command: &ApplicationCommandInteraction, ctx: &Context) -> Result<bool> {
    match command.data.name.as_str() {
        "ping" => ping::run(start_time, command, ctx).await,
        "roll" | "r" => roll::run(command, ctx).await,
        _ => return Ok(false),
    }.map(|_| true)
}