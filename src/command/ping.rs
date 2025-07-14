use std::collections::HashMap;

use eyre::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::async_trait;
use crate::{archmage::{ArchmageContext, Spell}, command::reply};

pub struct Ping;

#[async_trait]
impl Spell for Ping {
    fn register() -> Result<Vec<CreateApplicationCommand>> {
        let mut c = CreateApplicationCommand(HashMap::new());
        let _ = c.name("ping")
            .description("Determine server command (not network) latency");

        Ok(vec![c])
    }

    
    async fn run(ctx: ArchmageContext) -> Result<(), crate::archmage::ArchmageError> {
        let (time_start, ctx, command) = (ctx.start_time, ctx.ctx, ctx.command);
        let time_end = chrono::Utc::now().naive_utc();

        command.create_interaction_response(&ctx.http, |r| {
            reply()
            .title(format!(
                "Pong! ({}ms)",
                (time_end.time() - time_start.time()).num_milliseconds()
            ))
            .desc(format!(
                "Received {} UTC",
                time_start.format("%Y-%m-%d %H:%M:%S%.6f")
            ))
            .finish(r)
        }
        ).await?;
        Ok(())
    }
}
