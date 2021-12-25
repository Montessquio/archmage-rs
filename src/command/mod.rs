use serenity::client::Context;
use serenity::model:: channel::Message;
use tracing::{span, Level};
use regex::Regex;

use crate::database::MageDB;

mod roll;
mod config;

pub struct CommandHandler {

}

impl CommandHandler {
    pub fn new() -> CommandHandler {
        CommandHandler{}
    }

    pub async fn dispatch(&self, text: String, ctx: &Context, msg: &Message, db: &MageDB) {
        let span = span!(Level::INFO, "command dispatch");
        let _guard = span.enter();

        let args = message_to_args(&text);

        match args[0].to_lowercase().as_str() {
            // Handle config and its subcommands.
            "config" => config::config_handler(ctx, msg, args, db).await,

            // The roll command.
            "roll" => roll::roll_handler(ctx, msg, args).await,

            // Account for the !<dice_expr> shorthand.
            _ => {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r#"^\(*\d*d\d+"#).unwrap();
                }
                if RE.is_match(&args[0]) {
                    // Run the roll handler.
                    roll::roll_handler(ctx, msg, args).await;
                }
            },
        };
    }
}

// Splits the arguments string by spaces, but preserves quoted groups.
fn message_to_args(message: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"[^\s"']+|("[^"]*")|('[^']*')"#).unwrap();
    }
    return RE.find_iter(message).map(|m| m.as_str().to_owned()).collect();
}