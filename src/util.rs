//! Utility functions for use in all other source files.
use serenity::model::guild::Member;
use serenity::model::id::{UserId, GuildId};
use serenity::{client::Context, model::id::ChannelId};
use serenity::model:: channel::{Message, GuildChannel};
use serenity::model::permissions::Permissions;

// Get the GuildChannel that corresponds to a given id.
#[inline]
pub async fn channel_from_id(ctx: &Context, channel_id: ChannelId) -> Result<GuildChannel, String> {
    match ctx.cache.guild_channel(channel_id).await {
        Some(channel) => Ok(channel),
        None => Err("Could not retrieve guild's channel data.".to_owned()),
    }
}

// Get the member associated with a given author id and a guild.
#[inline]
pub async fn member_in_channel(ctx: &Context, guild_id: GuildId, member_id: UserId) -> Result<Member, String> {
    match ctx.cache.member(guild_id, member_id).await {
        Some(member) => Ok(member),
        None =>  Err("Could not retrieve member data.".to_owned()),
    }
}

/// Checks if a given user ID has a set of permissions in the given channel ID.
pub async fn member_has_permissions(ctx: &Context, msg: &Message, perms: Permissions) -> Result<bool, String> {
    match channel_from_id(ctx, msg.channel_id).await {
        Err(e) => return Err(e),
        Ok(channel) => match member_in_channel(ctx, channel.guild_id, msg.author.id).await {
            Err(e) => return Err(e),
            Ok(member) => {
                let member_permissions = {
                    match msg.guild_id {
                        Some(guild_id) => match ctx.cache.guild(guild_id).await {
                            Some(guild) => match guild.user_permissions_in(&guild.channels[&msg.channel_id], &member) {
                                Ok(permissions) => permissions,
                                Err(e) => return Err(format!("Error finding user role data: {}", e)),
                            },
                            None => return Err("Error finding user role data (Code 2)".to_owned()),
                        },
                        None => return Err("Error finding user role data (Code 3)".to_owned()),
                    }
                };

                if member_permissions.contains(perms) {
                    return Ok(true);
                }
                return Ok(false);
            },
        },
    };
}

/*

    
*/