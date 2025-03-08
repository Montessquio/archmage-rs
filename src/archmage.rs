use crate::command::{ArchmageCommand, CommandDispatcher};
use crate::Config;
use eyre::{bail, eyre, Result};
use serde::Serialize;
use serenity::all::{CreateEmbed, CreateInteractionResponseMessage};
use serenity::async_trait;
use serenity::model::prelude::Ready;
use serenity::{model::prelude::*, prelude::*};
use tracing::{event, span, Level};

#[non_exhaustive]
pub struct Archmage {
    config: Config,
    commands: CommandDispatcher,
}

impl Archmage {
    /// Create a new *Archmage* instance with a given configuration.
    /// Typically you want to start the bot by using [Archmage::start]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            commands: CommandDispatcher::new(),
        }
    }

    /// Add new commands to the bot. This method accepts any 
    /// [crate::command::ArchmageCommand] as its type argument, which may contain
    /// one or more Archmage commands. For more information on defining new
    /// commands to pass to this method, see the ArchmageCommand trait documentation.
    /// 
    /// This version is for use with the builder pattern in scenarios where the
    /// command list is known or checked ahead of time. It will panic if a command
    /// is redefined. If you need a fallible version, use [Archmage::try_add_commands].
    pub fn with_commands<T>(mut self) -> Self where T: ArchmageCommand {
        self.try_add_commands::<T>().unwrap();
        self
    }

    /// Add new commands to the bot. This method accepts any 
    /// [crate::command::ArchmageCommand] as its type argument, which may contain
    /// one or more Archmage commands. For more information on defining new
    /// commands to pass to this method, see the ArchmageCommand trait documentation.
    pub fn try_add_commands<T>(&mut self) -> Result<()> where T: ArchmageCommand {
        self.commands.register::<T>()
    }

    /// Launch the bot, connect to Discord and listen for events.
    pub async fn start(self) -> Result<()> {
        let config = self.config.clone();
        let mut client = Client::builder(config.token, GatewayIntents::all())
            .application_id(config.appid.into())
            .event_handler(self)
            // TODO: Set Voice Handler
            .await
            .expect("Could not build client!");

        if let Err(why) = client.start().await {
            bail!("An error occurred while running the client: {:?}", why);
        }
        Ok(())
    }
}

/// The entry point for all incoming Discord events!
/// If you're looking to trace the path incoming data takes through the bot,
/// start here. Each method here corresponds roughly to one incoming message type
/// from Discord. They are then routed to the impl block below, which specializes
/// interactions and provides helpers when needed.
#[async_trait]
impl EventHandler for Archmage {
    /// Runs whenever a bot interaction is called for.
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction.clone() {
            Interaction::Command(command) => {
                if let Err(e) = self.handle_command(&command, &ctx).await {
                    self.send_terminal_error_message(
                        command,
                        e,
                        &ctx,
                        interaction.message_component().map(|m| m.message),
                    ).await;
                }
            }
            _ => {} // Unimplemented Interactions should be ignored, for now. Other interaction types can be added here, later.
        };
    }

    /// Runs when the bot starts up and is connected to discord.
    async fn ready(&self, ctx: Context, event: Ready) {
        let span = span!(Level::INFO, "event ready");
        let _guard = span.enter();

        // Ensure only allowed guilds have the bot
        for guild in event.guilds {
            self.join_guild(&guild.id, &ctx).await
        }
    }

    /// Runs when the bot joins a new guild.
    async fn guild_create(&self, ctx: Context, guild: Guild, _: Option<bool>) {
        let span = span!(Level::INFO, "event guild_create");
        let _guard = span.enter();

        self.join_guild(&guild.id, &ctx).await
    }

    /// Runs when a thread is created.
    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        let span = span!(Level::ERROR, "thread_enter");
        let _guard = span.enter();
        if let Err(error) = thread.id.join_thread(ctx).await {
            event!(Level::ERROR, "Error joining thread: {}", error)
        }
    }
}

/// Helper functions and specializations for handlers above.
impl Archmage {
    /// Called by any event that results in a connection to a guild.
    /// If the guild is allowed, registers guild commands.
    /// If the guild is not allowed, leaves the guild.
    async fn join_guild(&self, guild_id: &GuildId, ctx: &Context) {
        if let Err(e) = self.leave_if_not_allowed(guild_id, ctx).await {
            event!(
                Level::WARN,
                "Error leaving illegal Guild '{}': {}",
                guild_id,
                e
            );
        } else {
            // Runs if the guild is allowed.
            // Inform the guild of supported commands.
            if let Err(e) = self.register_commands(guild_id, ctx).await {
                event!(
                    Level::ERROR,
                    "Error registering commands for guild '{}': {}",
                    guild_id,
                    e
                );
            }
            event!(Level::INFO, "Joined Guild {}", guild_id);
        }
    }

    /// Leaves the provided guild if it's not in the allowed list.
    /// Returns `Ok(())` whether or not the guild was in the allowed list, and
    /// only returns `Err(Report)` when there was some upstream error.
    async fn leave_if_not_allowed(&self, guild: &GuildId, ctx: &Context) -> Result<()> {
        if !self.config.allowed_guilds.contains(&guild.get()) {
            event!(Level::WARN, "Disconnecting from illegal guild: {}", guild);
            if let Err(error) = guild.leave(&ctx).await {
                bail!("Error leaving guild '{}': {}", guild.get(), error)
            }
        }
        Ok(())
    }

    /// Registers bot commands for the given build. Should be called once on guild join.
    pub async fn register_commands(&self, guild: &GuildId, ctx: &Context) -> Result<()> {
        if let Err(e) = guild
            .set_commands(ctx.http(), self.commands.get_all_defs().cloned().collect())
            .await
        {
            return Err(
                eyre!(e).wrap_err(format!("Unable to set guild commands for guild '{guild}!'"))
            );
        }
        Ok(())
    }

    /// Specialization sub-function of [Archmage::interaction_create].
    /// Handles slash commands, specifically.
    ///
    /// This function largely delegates the actual command dispatch, i.e. deciding
    /// which command to run, to [CommandDispatcher::run]. If you're looking for
    /// how to add new commands, check out the crate root module `main.rs`.
    pub async fn handle_command(&self, command: &CommandInteraction, ctx: &Context) -> Result<()> {
        match self
            .commands
            .run(&command.data.name, self, command, ctx)
            .await
        {
            Some(Ok(())) => Ok(()),
            Some(Err(e)) => Err(e),
            None => self.handle_unimplemented(command, ctx).await,
        }
    }

    /// Special error handler specifically for when a slash command has no handler
    /// associated with it. Since Discord traditionally does not allow slash commands
    /// that haven't been registered, this function is typically only called when
    /// a WIP command is added to Discord but isn't yet implemented.
    ///
    /// It prints a message to the channel the command was invoked in containing
    /// a user-friendly error message, and skips logging the interaction to the
    /// console.
    pub async fn handle_unimplemented(
        &self,
        command: &CommandInteraction,
        ctx: &Context,
    ) -> Result<()> {
        let response = command
            .create_response(
                &ctx.http,
                serenity::all::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().embed(
                        CreateEmbed::new()
                            .color(Color::from_rgb(0x00, 0xFF, 0x00))
                            .description(
                                "Archmage is still working on this spell! Please try again later.",
                            )
                            .title("Not yet implemented!")
                            .timestamp(Timestamp::now()),
                    ),
                ),
            )
            .await;

        if let Err(e) = response {
            event!(
                Level::ERROR,
                error = &format!("{}", e).as_str(),
                "DOUBLE FAULT! Error sending error message to user channel"
            )
        }

        Ok(())
    }

    /// Send a message to both the console and the appropriate channel. This
    /// method embeds a unique ID for each error, which allows channel errors to
    /// be matched to their more technical, verbose console errors.
    pub async fn send_terminal_error_message<CTX: Serialize>(
        &self,
        error_context: CTX,
        error: eyre::Report,
        discord_context: &Context,
        triggering_message: Option<impl AsRef<Message>>,
    ) {
        let err_id = uuid::Uuid::new_v4().as_simple().to_string();
        let env = serde_json::to_string(&error_context).expect("JSON Serialization Failure");
        event!(
            Level::ERROR,
            environment = &env.as_str(),
            error = &format!("{}", error).as_str(),
        );

        if let Some(message) = triggering_message {
            let response = message.as_ref()
            .channel_id.send_message(
                &discord_context.http,
                serenity::all::CreateMessage::new().add_embed(
                    CreateEmbed::new()
                        .color(Color::from_rgb(0xFF, 0x00, 0x00))
                        .description(format!("Artifices failed, magic gone awry. Something is wrong in the Archmage's tower! (Your error code is {})", &err_id))
                        .title("An Error Occurred")
                        .timestamp(Timestamp::now())
                    )
                ).await;

            if let Err(e) = response {
                event!(
                    Level::ERROR,
                    error = &format!("{}", e).as_str(),
                    "DOUBLE FAULT! Error sending error message to user channel"
                )
            }
        }
    }
}

unsafe impl Send for Archmage {}
unsafe impl Sync for Archmage {}