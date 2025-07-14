use chrono::Duration;
use futures::StreamExt;
use serenity::model::id::{ChannelId, GuildId};
use sqlx::{PgConnection, Postgres};
use tracing::{event, Level};

use crate::{
    command::pbp::{ChannelFlags, IntervalFlags, PbpInterval, PbpSchedule},
    db::DBError,
};

impl super::ArchmageDatabase {
    pub async fn get_pbp(
        &self,
        guild: GuildId,
        channel: ChannelId,
    ) -> Result<Option<PbpSchedule>, DBError> {
        let mut txn = self.pool.begin().await?;
        let r = get_pbp_schedule(&mut *txn, guild, channel).await?;
        txn.commit().await?;
        Ok(r)
    }

    /// Deletes the play-by-post schedule for the given channel.
    /// If there is no matching schedule, it silently succeeds and returns None.
    /// If there is a matching schedule, it returns the deleted schedule.
    pub async fn delete_pbp(
        &self,
        guild: GuildId,
        channel: ChannelId,
    ) -> Result<Option<PbpSchedule>, DBError> {
        let mut txn = self.pool.begin().await?;
        let r = delete_pbp_schedule(&mut *txn, guild, channel).await?;
        txn.commit().await?;
        Ok(r)
    }

    /// Inserts a play-by-post schedule for a given channel
    /// into the database. If this would replace an old one,
    /// it returns the old one.
    pub async fn set_pbp(&self, pbp: &PbpSchedule) -> Result<Option<PbpSchedule>, DBError> {
        let mut txn = self.pool.begin().await?;
        let r = set_pbp_schedule(&mut *txn, pbp).await?;
        txn.commit().await?;
        Ok(r)
    }
}

async fn get_pbp_schedule(
    db: &mut PgConnection,
    guild: GuildId,
    channel: ChannelId,
) -> Result<Option<PbpSchedule>, DBError> {
    let row = sqlx::query_as::<Postgres, (i64, i64, i64)>(
        r#"
        SELECT
            current_turn_index, 
            seconds_remaining, 
            flags
        FROM public.pbp_state 
        WHERE guild_id = $1 AND channel_id = $2"#,
    )
    .bind(guild.0 as i64)
    .bind(channel.0 as i64)
    .fetch_optional(&mut *db)
    .await?;

    let (current_turn_index, seconds_remaining, flags) = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    let mut ct = PbpSchedule {
        guild: guild.0,
        channel: channel.0,
        flags: ChannelFlags::from_bits_truncate(flags),
        current_turn_index,
        time_remaining: Duration::seconds(seconds_remaining),
        intervals: Vec::with_capacity(16),
    };

    let mut stream = sqlx::query_as::<Postgres, (i64, i64, i64, i64)>(
        r#"
        SELECT 
            user_id, 
            turn_index, 
            flags, 
            delay_secs
        FROM public.pbp_interval 
        WHERE guild_id = $1 AND channel_id = $2
        ORDER BY turn_index ASC"#,
    )
    .bind(guild.0 as i64)
    .bind(channel.0 as i64)
    .fetch(&mut *db);

    while let Some(row) = stream.next().await {
        let (user_id, turn_index, flags, delay_secs) = row?;
        if turn_index != ct.intervals.len() as i64 {
            event!(
                Level::WARN,
                guild = guild.0,
                channel = channel.0,
                user = user_id,
                index_in_db = turn_index,
                expected_index = ct.intervals.len(),
                "Index Mismatch"
            );
        }
        ct.intervals.push(PbpInterval {
            user: user_id as u64,
            flags: IntervalFlags::from_bits_truncate(flags),
            delay: Duration::seconds(delay_secs),
        })
    }

    if ct.intervals.is_empty() {
        event!(
            Level::WARN,
            guild = guild.0,
            channel = channel.0,
            "PBP was enabled but had no intervals"
        );
    }

    Ok(Some(ct))
}

/// Deletes the play-by-post schedule for the given channel.
/// If there is no matching schedule, it silently succeeds and returns None.
/// If there is a matching schedule, it returns the deleted schedule.
async fn delete_pbp_schedule(
    db: &mut PgConnection,
    guild: GuildId,
    channel: ChannelId,
) -> Result<Option<PbpSchedule>, DBError> {
    let existing_pbp = get_pbp_schedule(&mut *db, guild, channel).await?;

    // ON DELETE CASCADE will take care of the intervals for us.
    let _ = sqlx::query(
        r#"
        DELETE FROM public.pbp_state 
        WHERE guild_id = $1 AND channel_id = $2"#,
    )
    .bind(guild.0 as i64)
    .bind(channel.0 as i64)
    .execute(&mut *db)
    .await?;

    Ok(existing_pbp)
}

/// Inserts a play-by-post schedule for a given channel
/// into the database. If this would replace an old one,
/// it returns the old one.
async fn set_pbp_schedule(
    db: &mut PgConnection,
    pbp: &PbpSchedule,
) -> Result<Option<PbpSchedule>, DBError> {
    let old_pbp = delete_pbp_schedule(&mut *db, GuildId(pbp.guild), ChannelId(pbp.channel)).await?;

    let _ = sqlx::query(
        r#"
        INSERT INTO public.pbp_state (
            guild_id,
            channel_id,
            current_turn_index,
            seconds_remaining,
            flags
        )
        VALUES ($1, $2, $3, $4, $5)
    "#,
    )
    .bind(pbp.guild as i64)
    .bind(pbp.channel as i64)
    .bind(pbp.current_turn_index)
    .bind(pbp.time_remaining.num_seconds())
    .bind(pbp.flags.bits())
    .execute(&mut *db)
    .await?;

    let _ = sqlx::QueryBuilder::new(
        r#"
        INSERT INTO public.pbp_interval (
            guild_id, 
            channel_id, 
            user_id, 
            turn_index, 
            flags, 
            delay_secs
        )
        "#,
    )
    .push_values(
        pbp.intervals.iter().enumerate(),
        |mut b, (index, interval)| {
            let _ = b
                .push_bind(pbp.guild as i64)
                .push_bind(pbp.channel as i64)
                .push_bind(interval.user as i64)
                .push_bind(index as i32)
                .push_bind(interval.flags.bits())
                .push_bind(interval.delay.num_seconds());
        },
    )
    .build()
    .execute(&mut *db)
    .await?;

    Ok(old_pbp)
}