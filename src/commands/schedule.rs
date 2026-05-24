use crate::db::assert_found;
use crate::error::SchedulerError;
use crate::permissions::{channel_id_from_str, is_admin, require_admin_sync};
use crate::time::format_interval;
use crate::{Context, Error};

use super::{author_id_string, author_name, guild_id_string, reply_ephemeral};

#[poise::command(slash_command)]
pub async fn schedule(
    ctx: Context<'_>,
    #[description = "Numero ottenuto da /ss overlaps eseguito da admin"] id: i64,
) -> Result<(), Error> {
    let admin = is_admin(ctx).await?;
    require_admin_sync(admin)?;

    let guild_id = guild_id_string(ctx)?;
    let item = assert_found(
        ctx.data()
            .db
            .latest_valid_overlap_item(&guild_id, id)
            .await?,
    )
    .map_err(|_| SchedulerError::MissingOverlapBatch)?;

    let participant_list = if item.participant_usernames.is_empty() {
        "nessun partecipante registrato".to_string()
    } else {
        item.participant_usernames.join(", ")
    };

    let public_message = format!(
        "🎲 **Prossima sessione fissata!**\n\nQuando: {}\nPartecipanti disponibili: {}\nSchedulata da: {}",
        format_interval(item.start_utc, item.end_utc),
        participant_list,
        author_name(ctx)
    );

    let channel_id = ctx
        .data()
        .db
        .schedule_channel(&guild_id)
        .await?
        .ok_or(SchedulerError::MissingScheduleChannel)?;
    let channel = channel_id_from_str(&channel_id).ok_or_else(|| {
        SchedulerError::InvalidInput("il canale configurato nel database non e' valido".to_string())
    })?;
    let sent = channel.say(ctx.http(), &public_message).await?;
    let message_id = Some(sent.id.to_string());
    let scheduled_by_user_id = author_id_string(ctx);
    let scheduled_by_username = author_name(ctx);

    let session_id = ctx
        .data()
        .db
        .insert_scheduled_session(
            &guild_id,
            &item.id,
            (&scheduled_by_user_id, &scheduled_by_username),
            &item,
            message_id
                .as_deref()
                .map(|message_id| (channel_id.as_str(), message_id)),
        )
        .await?;

    let response = format!(
        "Sessione `{}` schedulata: {}.",
        session_id,
        format_interval(item.start_utc, item.end_utc)
    );

    reply_ephemeral(ctx, response).await
}
