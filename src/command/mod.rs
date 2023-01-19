use serenity::{prelude::*, model::application::interaction::application_command::ApplicationCommandInteraction};
use serenity::model::prelude::*;
use eyre::{Result, anyhow};

use crate::archmage::Archmage;

mod roll;
mod ping;

impl Archmage {
    pub async fn register_commands_for_guild(&self, guild: &GuildId, ctx: &Context) -> Result<()> {
        let _ = GuildId::set_application_commands(guild, &ctx.http, |commands| {
            commands
                .create_application_command(|command| ping::register(command))
                .create_application_command(|command| roll::register(command))
                .create_application_command(|command| roll::register_short(command))
        })
        .await?;
    
        Ok(())
    }

    pub async fn handle_command(&self, start_time: chrono::NaiveDateTime, command: &ApplicationCommandInteraction, ctx: &Context) -> Result<()> {
        match command.data.name.as_str() {
            "ping" => ping::run(start_time, command, &ctx).await,
            "roll" | "r" => roll::run(command, &ctx).await,
            _ => Err(anyhow!("Unimplemented Command!")),
        }
    }
}