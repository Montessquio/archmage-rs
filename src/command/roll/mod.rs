//! A dice and arithmetic parsing and rolling utility.
use eyre::{eyre, bail, Result};
use parser::DiceParser;
use serenity::all::{
    Color, CommandDataOptionValue, CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponseMessage, Timestamp
};
use serenity::prelude::*;

mod parser;

pub async fn run(command: &CommandInteraction, ctx: &Context) -> Result<()> {
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

pub fn register() -> CreateCommand {
    CreateCommand::new("roll")
        .description("Roll a die or calculate a value")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "expression",
                "A dice or calculator expression",
            )
            .required(true),
        )
}

pub fn register_short() -> CreateCommand {
    CreateCommand::new("r")
        .description("Roll a die or calculate a value")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "expression",
                "A dice or calculator expression",
            )
            .required(true),
        )
}

// RollHandler is a recursive descent dice and calculation expression parser.
async fn roll_handler(ctx: &Context, command: &CommandInteraction, input: &str) -> Result<()> {
    let mut parser = DiceParser::new(input)?;
    let expr = parser.expr();

    let (result, work) = expr.eval();

    if !parser.errors().is_empty() {
        bail!(parser.errors()[0].clone());
    }

    command
    .create_response(
        &ctx.http,
        serenity::all::CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().embed(
                CreateEmbed::new()
                    .color(Color::from_rgb(0x00, 0xFF, 0x00))
                    .description(input)
                    .field("Rolls", work, false)
                    .field("Result", result.to_string(), false)
                    .title(format!("{} Rolled {}", command.user.name, result))
                    .timestamp(Timestamp::now()),
            ),
        ))
        .await.map(|_| ()).map_err(|e| eyre!(e))
}
