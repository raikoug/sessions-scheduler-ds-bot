use chrono::Utc;
use poise::serenity_prelude as serenity;

use crate::db::assert_found;
use crate::permissions::{is_admin, mention_user};
use crate::time::{format_interval, instance_for_week, planning_week};
use crate::{Context, Error};

use super::{author_id_string, guild_id_string, reply_ephemeral};

#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "ID alfanumerico da 8 caratteri della disponibilità"] id: String,
) -> Result<(), Error> {
    let guild_id = guild_id_string(ctx)?;
    let author_id = author_id_string(ctx);
    let admin = is_admin(ctx).await?;

    let row = assert_found(
        ctx.data()
            .db
            .find_active_availability(&guild_id, &id)
            .await?,
    )?;
    if !admin && row.user_id != author_id {
        return reply_ephemeral(ctx, "Puoi rimuovere solo le tue disponibilità.").await;
    }

    let now = Utc::now();
    let week = planning_week(now)?;
    let mut conflicts = Vec::new();
    if let Some(instance) = instance_for_week(&row, week)? {
        conflicts = ctx
            .data()
            .db
            .scheduled_conflicts_for_removed_availability(
                &guild_id,
                &row.user_id,
                instance.start_utc,
                instance.end_utc,
            )
            .await?;
    }

    ctx.data()
        .db
        .soft_remove_availability(&guild_id, &id)
        .await?;

    let mut message = format!(
        "Disponibilità `{}` rimossa. Ho anche pulito gli overlap salvati.",
        id
    );

    if !conflicts.is_empty() {
        message.push_str("\n\n⚠️ Attenzione: questa disponibilità copriva una sessione già schedulata. Ho avvisato l'admin che l'ha fissata.");
        notify_scheduling_admins(ctx, &row.user_id, &conflicts).await?;
    }

    reply_ephemeral(ctx, message).await
}

async fn notify_scheduling_admins(
    ctx: Context<'_>,
    removed_user_id: &str,
    conflicts: &[crate::models::ScheduledSession],
) -> Result<(), Error> {
    for session in conflicts {
        let Ok(admin_id) = session.scheduled_by_user_id.parse::<u64>() else {
            continue;
        };
        let admin_user_id = serenity::UserId::new(admin_id);
        let dm = admin_user_id.create_dm_channel(ctx.http()).await;
        if let Ok(channel) = dm {
            let _ = channel
                .say(
                    ctx.http(),
                    format!(
                        "⚠️ Problema su una sessione schedulata: {} ha rimosso una disponibilità che copriva `{}`. Sessione: {}. Controlla `/ss overlaps` e ripianifica se serve.",
                        mention_user(removed_user_id),
                        session.id,
                        format_interval(session.start_utc, session.end_utc)
                    ),
                )
                .await;
        }
    }
    Ok(())
}
