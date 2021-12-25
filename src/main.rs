//!

#[deny(bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements ,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true)]

#[deny(missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results)]

use serde::Deserialize;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::model::{
    channel::{Message, GuildChannel}, 
    gateway::Ready, 
    guild::{Guild, GuildStatus}
};
use serenity::model::prelude::GuildStatus::*;
use tracing::{span, event, Level};
use std::fs;

#[macro_use]
extern crate lazy_static;

mod database;
use database::MageDB;

mod command;
use command::CommandHandler;

mod model;

lazy_static!{
    static ref CONFIG: Config = {
        toml::from_str(&fs::read_to_string("secret/config.toml").unwrap()).expect("Invalid Configuration")
    };
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Create the listener object.
    let bot = Archmage {
        // Open and migrate sqlite database.
        database: MageDB::open().await,
        handler: CommandHandler::new(),
    };

    let token = fs::read_to_string("secret/token").unwrap();
    let appid = fs::read_to_string("secret/appid").unwrap().trim().parse::<u64>().unwrap();

    let mut client = Client::builder(token.trim())
        .application_id(appid)
        .event_handler(bot)
        .intents(
            GatewayIntents::all()
        )
        .await
        .expect("Error creating client");

    event!(Level::INFO, "Strike the Earth!");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

struct Archmage {
    database: MageDB,
    handler: CommandHandler,
}

#[async_trait]
impl EventHandler for Archmage {
    // Runs when a new message is created.
    async fn message(&self, ctx: Context, msg: Message) {
        let span = span!(Level::INFO, "event message");
        let _guard = span.enter();

        // Do not respond to self!
        if msg.is_own(&ctx.cache).await {
            return;
        }

        // A None guild id means the message was not sent over the gateway.
        // We are not interested in these messages, so we can safely ignore it.
        if msg.guild_id.is_none() {
            return;
        }
        let guild_id = msg.guild_id.unwrap().0;

        // Get the current guild's prefix. Only respond to commands
        // with the correct prefix.
        if let Some(prefix) = self.database.get_guild_prefix(&guild_id).await {
            if let Some(message) = msg.content.strip_prefix(&prefix){
                // Delegate information to the command handler.
                self.handler.dispatch(message.to_owned(), &ctx, &msg, &self.database).await;
            }
        }
    }

    // Runs when the bot starts up and is connected to discord.
    async fn ready(&self, ctx: Context, event: Ready) {
        let span = span!(Level::INFO, "event ready");
        let _guard = span.enter();

        for guild in event.guilds {
            // If the newly joined guild is not in the allowed list, leave.
            if leave_if_not_allowed(&guild, &ctx).await {
                // Runs if the guild is allowed.
                event!(Level::INFO, "Joined Guild {}", guild.id());
                self.database.set_up_new_guild(&guild.id().0).await;
            }
        }
    }

    // Runs when the bot joins a new guild.
    async fn guild_create(&self, ctx: Context, guild: Guild, _: bool) {
        let span = span!(Level::INFO, "event guild_create");
        let _guard = span.enter();

        // If the newly joined guild is not in the allowed list, leave.
        if leave_if_not_allowed(&GuildStatus::OnlineGuild(guild.clone()), &ctx).await {
            // Runs if the guild is allowed.
            event!(Level::INFO, "Joined Guild {}", guild.id.0);
            self.database.set_up_new_guild(&guild.id.0).await;
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

// Leaves the provided guild if it's not in the allowed list.
// Returns whether or not the guild was in the allowed list.
async fn leave_if_not_allowed(guild: &GuildStatus, ctx: &Context) -> bool {
    // Helper function for below.
    #[inline]
    fn is_allowed_guild(id: &u64) -> bool {
        for gd in &CONFIG.allowed_guilds {
            if id == gd {
                return true;
            }
        }
        false
    }

    match guild {
        OnlinePartialGuild(g) => {
            // If the newly joined guild is not in the allowed list, leave.
            if !is_allowed_guild(&g.id.0) {
                event!(Level::WARN, "Connected to illegal guild: {}", g.id.0);
                if let Err(error) = g.leave(&ctx).await {
                    event!(Level::ERROR, "Error leaving guild: {}", error)
                }
                return false;
            }
            true
        },
        OnlineGuild(g) => {
            // If the newly joined guild is not in the allowed list, leave.
            if !is_allowed_guild(&g.id.0) {
                event!(Level::WARN, "Connected to illegal guild: {}", g.id.0);
                if let Err(error) = g.leave(&ctx).await {
                    event!(Level::ERROR, "Error leaving guild: {}", error)
                }
                return false;
            }
            true
        },
        // Can't do anything with offline guilds.
        _ => false
    }
}

#[derive(Deserialize)]
struct Config {
    pub allowed_guilds: Vec<u64>,
}