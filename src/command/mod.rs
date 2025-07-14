mod ping;
pub use ping::Ping;
mod roll;
pub use roll::{Roll, RollError};

pub mod pbp;
pub use pbp::PbpCommand;

use serenity::{builder::CreateInteractionResponse, json::Value, model::prelude::{command::CommandOptionType, interaction::{application_command::{CommandData, CommandDataOption}, InteractionResponseType}}, utils::Color};
pub struct StandardResponse {
    color: Color,
    title: String,
    description: String,
    fields: Vec<(String, String, bool)>,
}

fn reply() -> StandardResponse {
    StandardResponse {
        color: Color::from_rgb(0x00, 0xFF, 0x00),
        title: String::new(),
        description: String::new(),
        fields: Vec::new(),
    }
}

impl StandardResponse {
    #[allow(unused)]
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    pub fn title(&mut self, title: impl ToString) -> &mut Self {
        self.title = title.to_string();
        self
    }

    pub fn desc(&mut self, desc: impl ToString) -> &mut Self {
        self.description = desc.to_string();
        self
    }

    pub fn field(&mut self, title: &str, value: &str, inline: bool) -> &mut Self {
        self.fields.push((title.to_owned(), value.to_owned(), inline));
        self
    }

    pub fn finish<'a, 'b>(&mut self, response: &'a mut CreateInteractionResponse<'b>) -> &'a mut CreateInteractionResponse<'b> {
        response
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|m| {
            m.embed(|e| {
                e.color(self.color)
                    .description(self.description.clone())
                    .title(self.title.clone())
                    .fields(self.fields.clone())
                    .timestamp(chrono::Utc::now().to_rfc3339())
            })
        })
    }
}

pub trait CommandOptionsRetrievalExt {
    fn find<'a>(&'a self, name: &str) -> Option<&'a CommandDataOption>;
}

pub trait CommandOptionsInterpretationExt {
    fn interpret<'a>(&'a self, kind: CommandOptionType) -> Option<&'a Value>;
}

impl CommandOptionsRetrievalExt for Option<&CommandDataOption> {
    fn find<'a>(&'a self, name: &str) -> Option<&'a CommandDataOption> {
        self.and_then(|cdo| cdo.find(name))
    }
}


impl CommandOptionsInterpretationExt for Option<&CommandDataOption> {
    fn interpret(&self, kind: CommandOptionType) -> Option<&Value> {
        self.and_then(|cdo| cdo.interpret(kind))
    }
}

impl CommandOptionsInterpretationExt for CommandDataOption {
    fn interpret<'a>(&'a self, kind: CommandOptionType) -> Option<&'a Value> {
        match self.kind == kind {
            true => self.value.as_ref(),
            false => None,
        }
    }
}

impl CommandOptionsRetrievalExt for CommandDataOption {
    fn find<'a>(&'a self, name: &str) -> Option<&'a CommandDataOption> {
        self.options.iter().find(|cdo| cdo.name == name)
    }
}

impl CommandOptionsRetrievalExt for CommandData {
    fn find<'a>(&'a self, name: &str) -> Option<&'a CommandDataOption> {
        self.options.iter().find(|cdo| cdo.name == name)
    }
}