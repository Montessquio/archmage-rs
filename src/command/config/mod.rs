//! Handles interpretation of the !config command.
use serenity::client::Context;
use serenity::model:: channel::Message;
use serenity::model::prelude::ReactionType;
use serenity::model::permissions::Permissions;

use regex::Regex;
use crate::database::MageDB;

pub async fn config_handler(ctx: &Context, msg: &Message, mut args: Vec<String>, db: &MageDB) {
    // Remove leading "config" from the arguments.
    args.remove(0);

    if args.is_empty() {
        // Show config help.
    }

    match args[0].to_lowercase().as_str() {
        "prefix" => set_prefix(ctx, msg, args.get(1), db).await,

        "dm" => add_dm(ctx, msg, args, db).await,
        //"remove" => update_campaign(ctx, msg, args, db).await,

        // Account for the !<dice_expr> shorthand.
        _ => {
            // Unknown Command
        },
    };
}

// Change a server's command prefix.
async fn set_prefix(ctx: &Context, msg: &Message, prefix: Option<&String>, db: &MageDB) {
    if prefix.is_none() {
        return;
    }

    // Get the Member who sent the message.
    let member = {
        let channel = match ctx.cache.guild_channel(msg.channel_id).await {
            Some(channel) => channel,
            None => {
                if let Err(why) = msg.channel_id.say(ctx, "Error finding channel data").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            },
        };
    
        match ctx.cache.member(channel.guild_id, msg.author.id).await {
            Some(member) => member,
            None => {
                if let Err(why) = msg.channel_id.say(&ctx, "Error finding member data").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            },
        }
    };

    // Get all the roles in this guild.
    let member_permissions = {
        match msg.guild_id {
            Some(guild_id) => match ctx.cache.guild(guild_id).await {
                Some(guild) => match guild.user_permissions_in(&guild.channels[&msg.channel_id], &member) {
                    Ok(permissions) => permissions,
                    Err(e) => {
                        if let Err(why) = msg.channel_id.say(&ctx, "Error finding user role data (Code 1)").await {
                            println!("Error sending message: {:?}", why);
                        }
                        return;
                    },
                },
                None => {
                    if let Err(why) = msg.channel_id.say(&ctx, "Error finding user role data (Code 2)").await {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                }
            },
            None => {
                if let Err(why) = msg.channel_id.say(&ctx, "Error finding user role data (Code 3)").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }
        }
    };

    // Only change the prefix if the user whoo sent the command is an Administrator.
    if member_permissions.contains(Permissions::ADMINISTRATOR) {
        if let Some(guild) = msg.guild_id {
            if let Err(e) = db.set_prefix(guild.as_u64(), prefix.unwrap()).await {
                let _ = msg.channel_id.say(&ctx, format!("Internal Error: {}", e)).await;
            } else {
                if let Err(_) = msg.react(&ctx, ReactionType::Unicode("✅".to_string())).await {
                    let _ = msg.channel_id.say(&ctx, format!("Successfully set server prefix to `{}`", prefix.unwrap())).await;
                }
            }
        }
    }
}

// Create a new campaign if it does not exist, or add a DM to an existing one.
async fn add_dm(ctx: &Context, msg: &Message, mut args: Vec<String>, db: &MageDB) {
        // Remove leading "dm" from the arguments.
        args.remove(0);

        // User is first parameter, Campaign is second.
        let user_id = {
            // If they used a ping, then simply extract and validate the ID.
            lazy_static! {
                static ref RE: Regex = Regex::new(r#"^\(*\d*d\d+"#).unwrap();
            }

            let channel = match ctx.cache.guild_channel(msg.channel_id).await {
                Some(channel) => channel,
                None => {
                    if let Err(why) = msg.channel_id.say(ctx, "Error finding channel data").await {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                },
            };

            let members = match channel.members(&ctx.cache).await {
                Ok(m) => m,
                Err(e) => {
                    if let Err(why) = msg.channel_id.say(ctx, "Error finding member data").await {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                },
            };


        };
}