//! # Archmage Tabletop Automator
//!
//! *Archmage* is a chatbot, backed by Discord, designed to automate all your
//! tabletop RPG needs! It's primary design is meant to be *unobtrusive*, there
//! to help you when you need it, and invisible when you don't!
//!
//! If you're just trying to run Archmage yourself, you probably want to read
//! the main README.md page - the documentation here is primarily for developers
//! looking to improve Archmage!

// Refuse to compile unclean code.
#![deny(
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
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
    unused_variables
)]

use serde::Deserialize;
use std::fs;
use tracing::{event, Level};

mod archmage;
use archmage::Archmage;
mod command;

#[derive(Deserialize, Debug, Clone)]
struct Config {
    #[serde(alias = "allowed-guilds")]
    pub allowed_guilds: Vec<u64>,

    #[serde(alias = "app-id")]
    pub appid: u64,

    #[serde(alias = "secret")]
    pub token: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config = toml::from_str(
        &fs::read_to_string("secret/config.toml").expect("secret/config.toml not present"),
    )
    .expect("Invalid Configuration");

    event!(Level::INFO, "Strike the Earth!");

    Archmage::new(config)
        .with_commands::<(command::ping::PingCommand, command::roll::RollCommand)>()
        .start()
        .await
}
