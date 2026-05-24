use chrono::Utc;

use crate::overlap::compute_full_overlaps;
use crate::permissions::is_admin;
use crate::time::{format_interval, format_local_date};
use crate::{Context, Error};

use super::{author_id_string, author_name, guild_id_string, planning_instances, reply_ephemeral};

#[poise::command(slash_command)]
pub async fn overlaps(ctx: Context<'_>) -> Result<(), Error> {
    let now = Utc::now();
    let guild_id = guild_id_string(ctx)?;
    let (week, instances) = planning_instances(ctx, now).await?;
    let admin = is_admin(ctx).await?;
    let slots = compute_full_overlaps(&instances);

    if slots.is_empty() {
        return reply_ephemeral(
            ctx,
            "Non ci sono overlap completi: almeno una persona attiva nella settimana non è disponibile negli stessi slot degli altri.",
        )
        .await;
    }

    if admin {
        ctx.data()
            .db
            .store_overlap_batch(
                &guild_id,
                &week.start_date.to_string(),
                &author_id_string(ctx),
                &author_name(ctx),
                &slots,
            )
            .await?;
    }

    let mut lines = vec![format!(
        "Overlap completi dal {} al {}:",
        format_local_date(week.start_date),
        format_local_date(week.end_date - chrono::Duration::days(1))
    )];
    lines.push(String::new());

    for slot in &slots {
        if admin {
            lines.push(format!(
                "{}) {}",
                slot.display_index,
                format_interval(slot.start_utc, slot.end_utc)
            ));
        } else {
            lines.push(format!(
                "- {}",
                format_interval(slot.start_utc, slot.end_utc)
            ));
        }
    }

    if admin {
        lines.push(String::new());
        lines.push("Ho salvato questi ID per `/ss schedule id:[numero]`. Qualsiasi modifica alle disponibilità li invaliderà.".to_string());
    }

    reply_ephemeral(ctx, lines.join("\n")).await
}
