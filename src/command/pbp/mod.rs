use std::collections::HashMap;
use std::fmt::{Display, Write};

use chrono::Duration;
use eyre::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::async_trait;
use serenity::model::prelude::command::CommandOptionType;
use thiserror::Error;
use crate::archmage::{ArchmageContext, ArchmageError, Spell};
use crate::command::DurationDiscordPrintExt;

mod set_pbp;
mod get_pbp;
mod set_pbp_turn;
mod skip_pbp;
mod set_pbp_time;
mod delete_pbp;
mod pause_pbp;
mod resume_pbp;

#[derive(Error, Debug)]
pub enum PbpError {
    #[error("Parse Error: {0}")]
    ParseError(String),
}

pub struct PbpCommand;

#[async_trait]
impl Spell for PbpCommand {
    fn register() -> Result<Vec<CreateApplicationCommand>> {
        let mut c = CreateApplicationCommand(HashMap::new());
        let _ = c.name("pbp")
            .description("Control play-by-post for this channel.")
            .create_option(|o| o
                .name("set")
                .description("Set the play-by-post string for the channel.")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|o| o
                    .name("pbp-string")
                    .description("The pbp format string to set.")
                    .required(true)
                    .kind(CommandOptionType::String)
                )
            )
            .create_option(|o| o
                .name("get")
                .description("Get the play-by-post string for the channel.")
                .kind(CommandOptionType::SubCommand)
            )
            .create_option(|o| o
                .name("set-turn")
                .description("Get the play-by-post string for the channel.")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|o| o
                    .name("user")
                    .description("The turn index to change to.")
                    .kind(CommandOptionType::Integer)
                    .required(true)
                )
            )
            .create_option(|o| o
                .name("set-time")
                .description("Get the play-by-post string for the channel.")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|o| o
                    .name("seconds")
                    .description("How long to set the current turn timer to, in seconds. One-time override.")
                    .kind(CommandOptionType::Integer)
                    .required(true)
                )
            )
            .create_option(|o| o
                .name("delete")
                .description("Disable and remove play-by-post for this channel.")
                .kind(CommandOptionType::SubCommand)
            )
            .create_option(|o| o
                .name("pause")
                .description("Pause play-by-post timers for this channel.")
                .kind(CommandOptionType::SubCommand)
            )
            .create_option(|o| o
                .name("resume")
                .description("Resume play-by-post timers for this channel.")
                .kind(CommandOptionType::SubCommand)
            )
            .create_option(|o| o
                .name("skip")
                .description("Skip the current number of turns in the current channel.")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|o| o
                    .name("n")
                    .description("Number of turns to skip.")
                    .required(true)
                    .kind(CommandOptionType::Integer)
                )
            );

        Ok(vec![c])
    }

    
    async fn run(ctx: ArchmageContext) -> Result<(), ArchmageError> {
        let (_time_start, mut ctx, mut command, db) = (ctx.start_time, ctx.ctx, ctx.command, ctx.db);
        
        match command.data.options.first() {
            Some(cdo) => match cdo.kind {
                CommandOptionType::SubCommand => match cdo.name.as_str() {
                    "set" => set_pbp::run(&mut ctx, &mut command, &db).await?,
                    "get" => get_pbp::run(&mut ctx, &mut command, &db).await?,
                    "set-turn" => set_pbp_turn::run(&mut ctx, &mut command, &db).await?,
                    "set-time" => set_pbp_time::run(&mut ctx, &mut command, &db).await?,
                    "skip" => skip_pbp::run(&mut ctx, &mut command, &db).await?,
                    "delete" => delete_pbp::run(&mut ctx, &mut command, &db).await?,
                    "pause" => pause_pbp::run(&mut ctx, &mut command, &db).await?,
                    "resume" => resume_pbp::run(&mut ctx, &mut command, &db).await?,
                    _ => return Err(ArchmageError::InvalidSubCommand(cdo.name.clone())),
                },
                _ => return Err(ArchmageError::BadParamType("/subcommand".to_owned())),
            },
            None => return Err(ArchmageError::NotEnoughParams),
        };

        Ok(())
    }
}


bitflags::bitflags! {
    #[derive(PartialEq, PartialOrd, Copy, Clone, Debug, Default)]
    #[repr(transparent)]
    pub struct ChannelFlags: i64 {
        const PAUSED = 1 << 0;
        const ADMIN_PAUSED = 1 << 1;
    }
}

impl ChannelFlags {
    pub fn pretty_print(&self) -> String {
        let mut s = String::new();

        if self.contains(ChannelFlags::PAUSED) {
            s.push_str("PAUSED | ");
        }

        if self.contains(ChannelFlags::PAUSED) {
            s.push_str("ADMIN PAUSED | ");
        }

        let s = s.trim().trim_end_matches('|').to_owned();
        
        match s.is_empty() {
            true => "NONE".to_owned(),
            false => s,
        }
    }
}
 
bitflags::bitflags! {
    #[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
    #[repr(transparent)]
    pub struct IntervalFlags: i64 {
        /// Turn Remind if is 1, Turn End if is 0.
        const TURN_REMIND = 1 << 0;
    }
}

impl IntervalFlags {
    pub fn pretty_print(&self) -> String {
        let mut s = String::new();

        if self.contains(IntervalFlags::TURN_REMIND) {
            s.push_str("TURN REMINDER | ");
        }
        else {
            s.push_str("TURN END | ");
        }

        let s = s.trim().trim_end_matches('|').to_owned();

        match s.is_empty() {
            true => "NONE".to_owned(),
            false => s,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PbpSchedule {
    pub guild: u64,
    pub channel: u64,
    pub flags: ChannelFlags,
    pub current_turn_index: i64,
    pub time_remaining: Duration,
    pub intervals: Vec<PbpInterval>,
}

impl Display for PbpSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{g:{}, c:{}, f: \"", self.guild, self.channel)?;
        for (i, interval) in self.intervals.iter().enumerate() {
            write!(f, "{interval}")?;
            if i < self.intervals.len() - 1 {
                write!(f, ":")?;
            }
        }
        write!(f, 
            "!f{}!i{}!t{}\"}}",
            self.flags.bits(),
            self.current_turn_index,
            self.time_remaining,
        )
    }
}

impl PbpSchedule {
    pub fn parse_str(guild: u64, channel: u64, s: impl AsRef<str>) -> Result<Self, PbpError> {
        let s = s.as_ref().chars().filter(|c| !c.is_whitespace()).collect::<String>();
        let mut parts = s.split('!');

        let mut flags: Option<ChannelFlags> = None;
        let mut cti: Option<i64> = None;
        let mut time: Option<Duration> = None;

        let intervals = match parts.next() {    
            None => return Err(PbpError::ParseError(format!("Not enough PBP parts! Expected INTERVALS."))),
            Some(p) => p.split(":")
                                .enumerate()
                                .map(|(i, s)| PbpInterval::parse_str(s).map_err(|e| PbpError::ParseError(format!("Error parsing interval {i}: {e}"))))
                                .collect::<Result<Vec<PbpInterval>, PbpError>>()?,
        };

        for part in parts {
            match part.chars().next() {
                None => continue,
                Some('f') => match part.trim_start_matches('f').parse::<i64>() {
                    Err(e) => return Err(PbpError::ParseError(format!("Error parsing PBP global flags: {e}"))),
                    Ok(n) => match flags.is_some() {
                        true => return Err(PbpError::ParseError("Global flags specified more than once!".to_owned())),
                        false => flags = Some(ChannelFlags::from_bits_truncate(n)),
                    }
                },
                Some('i') => match part.trim_start_matches('i').parse::<i64>() {
                    Err(e) => return Err(PbpError::ParseError(format!("Error parsing PBP initial index: {e}"))),
                    Ok(n) => match flags.is_some() {
                        true => return Err(PbpError::ParseError("Initial index specified more than once!".to_owned())),
                        false => cti = n.checked_rem((intervals.len() - 1) as i64),
                    }
                },
                Some('t') => match part.trim_start_matches('t').parse::<i64>() {
                    Err(e) => return Err(PbpError::ParseError(format!("Error parsing PBP initial timer: {e}"))),
                    Ok(n) => match flags.is_some() {
                        true => return Err(PbpError::ParseError("Initial timer specified more than once!".to_owned())),
                        false => time = Some(Duration::seconds(n)),
                    }
                },
                Some(_) => return Err(PbpError::ParseError(format!("Unknown global field: \"{part}\""))),
            }
        }

        Ok(Self { 
            guild,
            channel, 
            flags: flags.unwrap_or_default(),
            current_turn_index: cti.unwrap_or(-1),
            time_remaining: time.unwrap_or_else(|| Duration::zero()),
            intervals
        })
    }

    pub fn pretty_print(&self) -> Result<String, ArchmageError> {
        let mut s = String::with_capacity(512);

        write!(
            s,
            "**Global Flags:** {}\n**Current Turn:** {}\n**Time Remaining:** {} ({}/{})\n**Turn Order:**\n",
            self.flags.pretty_print(),
            self.current_turn_index.to_string().as_str(),
            self.time_remaining.as_duration_pretty(),
            self.time_remaining.as_discord_date(),
            self.time_remaining.as_discord_span(),
        )?;

        for (index, interval) in self.intervals.iter().enumerate() {
            write!(
                s,
                "{}\n",
                interval.pretty_print(index as i64),
            )?;
        }

        Ok(s)
    }
}
 
#[derive(Debug, Clone)]
pub struct PbpInterval {
    pub user: u64,
    pub delay: Duration,
    pub flags: IntervalFlags,
}

impl Display for PbpInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}", 
            self.user, 
            self.delay.num_seconds() as u64, 
            self.flags.bits()
        )
    }
}

impl PbpInterval {
    pub fn parse_str(s: impl AsRef<str>) -> Result<Self, PbpError> {
        let mut s = s.as_ref().split('.');
        
        let user: u64 = match s.next().map(|s| s.trim_matches(['<', '>']).trim_matches('@').parse::<u64>()) {
            None => return Err(PbpError::ParseError(format!("Incomplete interval. Expected USER_ID."))),
            Some(Err(e)) => return Err(PbpError::ParseError(format!("Incomplete interval. Malformed USER_ID. e: {e}"))),         
            Some(Ok(n)) => n,
        };

        let delay: Duration = match s.next().map(|s| s.parse::<i64>()) {
            None => return Err(PbpError::ParseError(format!("Incomplete interval. Expected TIME."))),
            Some(Err(e)) => return Err(PbpError::ParseError(format!("Incomplete interval. Malformed TIME. e: {e}"))),         
            Some(Ok(n)) => Duration::seconds(n),
        };

        let flags: IntervalFlags = match s.next().map(|s| s.parse::<i64>()) {
            None => return Err(PbpError::ParseError(format!("Incomplete interval. Expected FLAGS."))),
            Some(Err(e)) => return Err(PbpError::ParseError(format!("Incomplete interval. Malformed FLAGS. e: {e}"))),         
            Some(Ok(n)) => IntervalFlags::from_bits_truncate(n),
        };

        return Ok(Self { user, delay, flags })
    }

    pub fn pretty_print(&self, index: i64) -> String {
        format!(
            "- {}: <@{}>\n  - Duration: {}\n  - Flags: {}",
            index,
            self.user,
            self.delay.as_duration_pretty(),
            self.flags.pretty_print(),
        )
    }
}