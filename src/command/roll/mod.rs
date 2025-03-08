//! A dice and arithmetic parsing and rolling utility.
use eyre::{bail, eyre, Result};
use parser::DiceParser;
use serenity::all::{
    Color, CommandDataOptionValue, CommandInteraction, CommandOptionType, CreateCommand,
    CreateCommandOption, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    Timestamp,
};
use serenity::prelude::*;

use crate::archmage::Archmage;

use super::{handle_fn, ArchmageCommand, HandleFn};

mod parser;

pub struct RollCommand;

impl ArchmageCommand for RollCommand {
    fn register() -> Vec<(String, CreateCommand, HandleFn)> {
        vec![
            (
                "roll".to_owned(),
                CreateCommand::new("roll")
                    .description("Roll a die or calculate a value")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "expression",
                            "A dice or calculator expression",
                        )
                        .required(true),
                    ),
                handle_fn!(Self::run),
            ),
            (
                "r".to_owned(),
                CreateCommand::new("r")
                    .description("Roll a die or calculate a value")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "expression",
                            "A dice or calculator expression",
                        )
                        .required(true),
                    ),
                handle_fn!(Self::run),
            )
        ]
    }
}

impl RollCommand {
    async fn run(_archmage: &Archmage, command: &CommandInteraction, ctx: &Context) -> Result<()> {
        let option = &command
            .data
            .options
            .first()
            .ok_or(eyre!("Expected dice or calculation expression"))?
            .value;

        match option {
            CommandDataOptionValue::String(input) => roll_handler(ctx, command, input).await,
            _ => bail!("Unexpected input type"),
        }
    }
}

// RollHandler is a recursive descent dice and calculation expression parser.
async fn roll_handler(ctx: &Context, command: &CommandInteraction, input: &str) -> Result<()> {
    let mut parser = match DiceParser::new(input) {
        Ok(p) => p,
        Err(e) => {
            return command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().embed(
                            CreateEmbed::new()
                                .color(Color::from_rgb(0xFF, 0x00, 0x00))
                                .description(e.to_string())
                                .title("An Error Occurred")
                                .timestamp(Timestamp::now()),
                        ),
                    ),
                )
                .await
                .map(|_| ())
                .map_err(|e| eyre!(e))
        }
    };
    let expr = parser.expr();

    let (result, work) = expr.eval();

    if !parser.errors().is_empty() {
        command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().embed(
                        CreateEmbed::new()
                            .color(Color::from_rgb(0xFF, 0x00, 0x00))
                            .description(parser.errors()[0].clone())
                            .title("An Error Occurred")
                            .timestamp(Timestamp::now()),
                    ),
                ),
            )
            .await
            .map(|_| ())
            .map_err(|e| eyre!(e))
    } else {
        command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().embed(
                        CreateEmbed::new()
                            .color(Color::from_rgb(0x00, 0xFF, 0x00))
                            .description(input)
                            .field("Rolls", work, false)
                            .field("Result", result.to_string(), false)
                            .title(format!("{} Rolled {}", command.user.name, result))
                            .timestamp(Timestamp::now()),
                    ),
                ),
            )
            .await
            .map(|_| ())
            .map_err(|e| eyre!(e))
    }
}
