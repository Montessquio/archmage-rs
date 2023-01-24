//!
#![deny(
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements ,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    unused_variables,
)]

use serde::Deserialize;
use serenity::{all::ApplicationId, client::Client};
use serenity::prelude::GatewayIntents;
use tracing::{event, Level};
use std::fs;

#[macro_use]
extern crate lazy_static;

mod archmage;
use archmage::Archmage;

mod command;

#[derive(Deserialize)]
struct Config {
    #[serde(alias = "allowed-guilds")]
    pub allowed_guilds: Vec<u64>,

    #[serde(alias = "app-id")]
    pub appid: u64,

    #[serde(alias = "secret")]
    pub token: String,
}

lazy_static!{
    static ref CONFIG: Config = {
        toml::from_str(&fs::read_to_string("secret/config.toml").expect("secret/config.toml not present")).expect("Invalid Configuration")
    };
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Create the listener object.
    let bot = Archmage::new();

    let mut client = Client::builder(CONFIG.token.trim(), GatewayIntents::all())
        .application_id(ApplicationId::new(CONFIG.appid))
        .event_handler(bot)
        .await
        .expect("Error creating client");

    event!(Level::INFO, "Strike the Earth!");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
