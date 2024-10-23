use eyre::{bail, Result};
use serenity::all::{CreateEmbed, CreateInteractionResponseMessage};
use serenity::async_trait;
use serenity::model::prelude::Ready;
use serenity::{model::prelude::*, prelude::*};
use tracing::{event, span, Level};

#[non_exhaustive]
pub struct Archmage {}

#[async_trait]
impl EventHandler for Archmage {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let start_time = chrono::Utc::now().naive_utc();

        if let Interaction::Command(command) = interaction {
            if let Err(e) = self.handle_command(start_time, &command, &ctx).await {
                let err_id = uuid::Uuid::new_v4().as_simple().to_string();
                let env = serde_json::to_string(&command).expect("JSON Serialization Failure");
                event!(
                    Level::ERROR,
                    environment = &env.as_str(),
                    error = &format!("{}", e).as_str(),
                );

                let response = command
                    .create_response(
                        &ctx.http,
                        serenity::all::CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().embed(
                                    CreateEmbed::new()
                                        .color(Color::from_rgb(0xFF, 0x00, 0x00))
                                        .description(format!("Artifices failed, magic gone awry. Something is wrong in the Archmage's tower! (Your error code is {})", &err_id))
                                        .title("An Error Occurred")
                                        .timestamp(Timestamp::now())
                    ))).await;

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

    // Runs when the bot starts up and is connected to discord.
    async fn ready(&self, ctx: Context, event: Ready) {
        let span = span!(Level::INFO, "event ready");
        let _guard = span.enter();

        // Ensure only allowed guilds have the bot
        for guild in event.guilds {
            if let Err(e) = Self::leave_if_not_allowed(&guild.id, &ctx).await {
                event!(
                    Level::WARN,
                    "Error leaving illegal Guild '{}': {}",
                    guild.id,
                    e
                );
            } else {
                // Runs if the guild is allowed.
                if let Err(e) = self.register_commands_for_guild(&guild.id, &ctx).await {
                    event!(
                        Level::ERROR,
                        "Error registering commands for guild '{}': {}",
                        guild.id,
                        e
                    );
                }
                event!(Level::INFO, "Joined Guild {}", guild.id);
            }
        }
    }

    // Runs when the bot joins a new guild.
    async fn guild_create(&self, ctx: Context, guild: Guild, _: Option<bool>) {
        let span = span!(Level::INFO, "event guild_create");
        let _guard = span.enter();

        // If the newly joined guild is not in the allowed list, leave.
        if let Err(e) = Self::leave_if_not_allowed(&guild.id, &ctx).await {
            event!(Level::ERROR, "{}", e);
        } else {
            // Runs if the guild is allowed. Register commands and continue.
            if let Err(e) = self.register_commands_for_guild(&guild.id, &ctx).await {
                event!(
                    Level::ERROR,
                    "Error registering commands for guild '{}': {}",
                    guild.id,
                    e
                );
            }
        }
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

impl Archmage {
    pub fn new() -> Self {
        Archmage {}
    }

    // Leaves the provided guild if it's not in the allowed list.
    // Returns whether or not the guild was in the allowed list.
    async fn leave_if_not_allowed(guild: &GuildId, ctx: &Context) -> Result<()> {
        let allowlist = &crate::CONFIG.allowed_guilds;
        if !allowlist.contains(&guild.get()) {
            event!(Level::WARN, "Disconnecting from illegal guild: {}", guild);
            if let Err(error) = guild.leave(&ctx).await {
                bail!("Error leaving guild '{}': {}", guild.get(), error)
            }
        }
        Ok(())
    }
}
