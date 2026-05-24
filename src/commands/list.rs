use chrono::Utc;

use crate::time::format_availability_row;
use crate::{Context, Error};

use super::{author_id_string, guild_id_string, reply_ephemeral};

#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = guild_id_string(ctx)?;
    let user_id = author_id_string(ctx);
    let rows = ctx
        .data()
        .db
        .list_user_future_availabilities(&guild_id, &user_id, Utc::now())
        .await?;

    if rows.is_empty() {
        return reply_ephemeral(ctx, "Non hai disponibilità future salvate.").await;
    }

    let mut lines = vec!["Le tue disponibilità future:".to_string(), String::new()];
    for row in rows {
        lines.push(format_availability_row(&row));
    }

    reply_ephemeral(ctx, lines.join("\n")).await
}
