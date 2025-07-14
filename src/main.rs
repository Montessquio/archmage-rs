//! Archmage, the discord bot! Behold its mighty magics.
//! 
//! This is the main module. It contains basic configuration
//! and launches the discord bot client.
#![deny(
    bad_style,
    //dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements ,
    patterns_in_fns_without_body,
    private_interfaces,
    private_bounds,
    unnameable_types,
    unconditional_recursion,
    //unused,
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
    //unused_results,
    unused_variables,
)]

use eyre::Result;
use serde::Deserialize;
use serenity::Client;
use serenity::prelude::GatewayIntents;
use tracing::{event, Level};
use std::fs;
use lazy_static::lazy_static;

mod archmage;
use archmage::Archmage;

use crate::db::ArchmageDatabase;
mod db;
mod command;
mod service;

#[derive(Deserialize)]
struct Config {
    #[serde(alias = "allowed-guilds")]
    pub allowed_guilds: Vec<u64>,

    #[serde(alias = "app-id")]
    #[serde(alias = "app_id")]
    pub appid: u64,

    #[serde(alias = "secret")]
    pub token: String,

    pub db_url: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub db_max_connections: u32,

    //pub global_admins: Vec<u64>,
}

lazy_static!{
    static ref CONFIG: Config = {
        toml::from_str(&fs::read_to_string("secret/config.toml").expect("secret/config.toml not present")).expect("Invalid Configuration")
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db = ArchmageDatabase::connect().await.expect("Database connection failed!");

    let mut bot = Archmage::new(db);
    bot.register_service::<service::PpbService>()?;

    bot.register_command::<command::Ping>()?;
    bot.register_command::<command::Roll>()?;
    bot.register_command::<command::PbpCommand>()?;

    let mut client = Client::builder(CONFIG.token.trim(), GatewayIntents::all())
        .application_id(CONFIG.appid)
        .event_handler(bot)
        .await
        .expect("Error creating client");

    event!(Level::INFO, "Strike the Earth!");

    // Start listening for events by starting a single shard
    if let Err(e) = client.start().await {
        println!("An error occurred while running the client: {:?}", e);
    }

    Ok(())
}
