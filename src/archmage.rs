//! Bot behavior entrypoint.
//!
//! This module sets up the event handler
//! struct `Archmange` which handles events
//! sent to it by Discord.

use crate::command::pbp::PbpError;
use crate::command::RollError;
use crate::db::ArchmageDatabase;
use crate::db::DBError;
use chrono::NaiveDateTime;
use eyre::{bail, Result};
use futures::future::BoxFuture;
use hashbrown::HashMap;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Ready;
use serenity::{model::prelude::*, prelude::*};
use tracing::warn;
use std::fmt::Display;
use std::sync::Arc;
use thiserror::Error;
use tracing::{event, span, Level};

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ArchmageError {
    #[error("Serenity Error: {0}")]
    Serenity(#[from] SerenityError),

    #[error("{0}")]
    Roll(#[from] RollError),

    #[error("DB Error: {0}")]
    DbError(#[from] DBError),

    #[error("PBP Error: {0}")]
    PbpError(#[from] PbpError),

    #[error("I/O Error: {0}")]
    Fmt(#[from] std::fmt::Error),

    #[error("Invalid Subcommand: {0}")]
    InvalidSubCommand(String),

    #[error("Not enough parameters!")]
    NotEnoughParams,

    #[error("Wrong param type! Path: {0}")]
    BadParamType(String),

    #[error("No guild!")]
    NoGuild,
}

unsafe impl Send for ArchmageError {}
unsafe impl Sync for ArchmageError {}

pub struct ArchmageContext {
    pub start_time: NaiveDateTime,
    pub db: Arc<ArchmageDatabase>,
    pub ctx: Context,
    pub command: ApplicationCommandInteraction,
}

pub type ArchmageResult = Result<(), ArchmageError>;
type ArchmageCallbackFn =
    Box<dyn Fn(ArchmageContext) -> BoxFuture<'static, ArchmageResult> + Send + Sync>;

pub struct Archmage {
    command_constructors: HashMap<String, CreateApplicationCommand>,
    handlers: HashMap<String, ArchmageCallbackFn>,
    db: Arc<ArchmageDatabase>,
}

impl Archmage {
    pub fn new(db: ArchmageDatabase) -> Self {
        Archmage {
            command_constructors: HashMap::new(),
            handlers: HashMap::new(),
            db: Arc::new(db),
        }
    }

    pub fn register_command<S: Spell + 'static>(&mut self) -> Result<()> {
        let commands = S::register()?;
        for command in commands {
            let name = match command.0.get("name") {
                None => bail!(format!("Provided command had no valid name: {command:#?}")),
                Some(n) => n,
            };
            event!(Level::INFO, "Registered Command \"{}\"", name.to_string().trim_matches('"'));

            if let Some(_) = self.handlers.insert(name.to_string().trim_matches('"').to_owned(), Box::new(S::run)) {
                warn!("Command '{}' has been overwritten in the registry!", command.0.get("name").unwrap())
            }
            // Not bothering to log this one because if the above went through then so did this one.
            let _ = self.command_constructors.insert(name.to_string(), command);
        }

        Ok(())
    }

    async fn join_guild(&self, guilds: impl Iterator<Item = GuildId>, ctx: Context) {
        let span = span!(Level::INFO, "event join_guild");
        let _guard = span.enter();

        for guild in guilds {
            // If the newly joined guild is not in the allowed list, leave.
            if let Err(e) = Self::leave_if_not_allowed(&guild, &ctx).await {
                event!(
                    Level::WARN,
                    "Error leaving illegal Guild '{}': {}",
                    guild.0,
                    e
                );
            } else {
                // Runs if the guild is allowed.
                if let Err(e) = self.register_commands_for_guild(&guild, &ctx).await {
                    event!(
                        Level::ERROR,
                        "Error registering commands for guild '{}': {}",
                        guild,
                        e
                    );
                }
                event!(Level::INFO, "Joined Guild {}", guild);
            }
        }
    }

    async fn register_commands_for_guild(&self, guild: &GuildId, ctx: &Context) -> Result<()> {
        let _ = GuildId::set_application_commands(guild, &ctx.http, |commands| {
            for (_, command) in &self.command_constructors {
                commands.add_application_command(command.clone());
            }
            commands
        })
        .await?;

        Ok(())
    }

    async fn handle_command(
        &self,
        start_time: NaiveDateTime,
        ctx: Context,
        command: ApplicationCommandInteraction,
    ) {
        let context = ArchmageContext {
            start_time,
            ctx: ctx.clone(),
            command: command.clone(),
            db: Arc::clone(&self.db),
        };

        match self.handlers.get(command.data.name.as_str()) {
            None => Self::not_found_message(ctx, command).await,
            Some(callback) => {
                if let Err(e) = callback(context).await {
                    Self::fail_message(ctx, command, e).await;
                }
            }
        };
    }

    pub async fn fail_message<E: Display>(
        ctx: Context,
        command: ApplicationCommandInteraction,
        error: E,
    ) {
        let err_id = uuid::Uuid::new_v4().as_simple().to_string();
        let env = serde_json::to_string(&command).expect("JSON Serialization Failure");
        event!(
            Level::ERROR,
            environment = &env.as_str(),
            error = &format!("{}", error).as_str(),
        );

        let response = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| {
                    m.embed(|e| {
                        use serenity::utils::Color;
                        e.color(Color::from_rgb(0x00, 0xFF, 0x00))
                            .description(format!("Artifices failed, magic gone awry. Something is wrong in the Archmage's tower! (Your error code is {})", &err_id))
                            .title("An Error Occurred")
                            .timestamp(chrono::Utc::now().to_rfc3339())
                    })
                })
            }
        ).await;

        if let Err(e) = response {
            event!(
                Level::ERROR,
                error = &format!("{}", e).as_str(),
                "DOUBLE FAULT! Error sending error message to user channel"
            )
        }
    }

    /*
    pub async fn unimplemented_message(ctx: Context, command: ApplicationCommandInteraction) {
        let response = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| {
                            m.embed(|e| {
                                use serenity::utils::Color;
                                e.color(Color::from_rgb(0x00, 0xFF, 0x00))
                                 .description("Archmage is still working on this spell! Please try again later.")
                                 .title("Not yet implemented!")
                                 .timestamp(chrono::Utc::now().to_rfc3339())
                            })
                        })
                    }
                ).await;

        if let Err(e) = response {
            event!(
                Level::ERROR,
                error = &format!("{}", e).as_str(),
                "DOUBLE FAULT! Error sending error message to user channel"
            )
        }
    }
     */

    pub async fn not_found_message(ctx: Context, command: ApplicationCommandInteraction) {
        let response = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| {
                        m.embed(|e| {
                            use serenity::utils::Color;
                            e.color(Color::from_rgb(0x00, 0xFF, 0x00))
                                .description("Archmage doesn't know this spell!.")
                                .title("Command Unknown!")
                                .timestamp(chrono::Utc::now().to_rfc3339())
                        })
                    })
            })
            .await;

        if let Err(e) = response {
            event!(
                Level::ERROR,
                error = &format!("{}", e).as_str(),
                "DOUBLE FAULT! Error sending error message to user channel"
            )
        }
    }

    // Leaves the provided guild if it's not in the allowed list.
    // Returns whether or not the guild was in the allowed list.
    async fn leave_if_not_allowed(guild: &GuildId, ctx: &Context) -> Result<()> {
        let allowlist = &crate::CONFIG.allowed_guilds;
        if !allowlist.contains(guild.as_u64()) {
            event!(Level::WARN, "Disconnecting from illegal guild: {}", guild);
            if let Err(error) = guild.leave(&ctx).await {
                bail!("Error leaving guild '{}': {}", guild.as_u64(), error)
            }
        }
        Ok(())
    }
}

#[async_trait]
pub trait Spell {
    /// Add the spell to Discord, and return a set of all command names
    /// this delegate responds to.
    fn register() -> Result<Vec<CreateApplicationCommand>>;
    async fn run(context: ArchmageContext) -> ArchmageResult;
}

#[async_trait]
impl EventHandler for Archmage {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let start_time = chrono::Utc::now().naive_utc();

        if let Interaction::ApplicationCommand(command) = interaction {
            self.handle_command(start_time, ctx, command).await;
        }
    }

    // Runs when the bot starts up and is connected to discord.
    async fn ready(&self, ctx: Context, event: Ready) {
        self.join_guild(event.guilds.iter().map(|g| g.id), ctx).await;
    }

    // Runs when the bot joins a new guild.
    async fn guild_create(&self, ctx: Context, guild: Guild, _: bool) {
        self.join_guild([guild.id].into_iter(), ctx).await;
    }

    // Runs when a thread is created.
    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        let span = span!(Level::ERROR, "thread_enter");
        let _guard = span.enter();
        if let Err(error) = thread.id.join_thread(ctx).await {
            event!(Level::ERROR, "Error joining thread: {}", error)
        }
    }
}
