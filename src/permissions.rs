use poise::serenity_prelude as serenity;

use crate::{Context, Error};

pub async fn is_admin(ctx: Context<'_>) -> Result<bool, Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(false);
    };

    let member = guild_id.member(ctx.http(), ctx.author().id).await?;
    let Some(guild) = ctx.guild() else {
        return Ok(false);
    };
    let Some(channel) = guild.channels.get(&ctx.channel_id()) else {
        return Ok(false);
    };

    Ok(guild.user_permissions_in(channel, &member).administrator())
}

pub fn require_admin_sync(is_admin: bool) -> Result<(), crate::error::SchedulerError> {
    if is_admin {
        Ok(())
    } else {
        Err(crate::error::SchedulerError::Forbidden)
    }
}

pub fn mention_user(user_id: &str) -> String {
    format!("<@{}>", user_id)
}

pub fn channel_id_from_str(value: &str) -> Option<serenity::ChannelId> {
    value.parse::<u64>().ok().map(serenity::ChannelId::new)
}
