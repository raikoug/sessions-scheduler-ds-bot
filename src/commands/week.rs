use std::collections::BTreeMap;

use chrono::{Datelike, Utc};

use crate::permissions::is_admin;
use crate::time::{format_interval, format_local_date, weekday_it, ROME};
use crate::{Context, Error};

use super::{planning_instances, reply_ephemeral};

#[poise::command(slash_command)]
pub async fn week(ctx: Context<'_>) -> Result<(), Error> {
    let now = Utc::now();
    let (planning_week, instances) = planning_instances(ctx, now).await?;
    let admin = is_admin(ctx).await?;

    if instances.is_empty() {
        return reply_ephemeral(
            ctx,
            "Nessuna disponibilità nella settimana di pianificazione.",
        )
        .await;
    }

    let mut lines = vec![format!(
        "Disponibilità dal {} al {}:",
        format_local_date(planning_week.start_date),
        format_local_date(planning_week.end_date - chrono::Duration::days(1))
    )];

    if admin {
        let mut by_day: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for instance in instances {
            let start = instance.start_utc.with_timezone(&ROME);
            let key = format!(
                "{} {}",
                weekday_it(start.weekday()),
                format_local_date(start.date_naive())
            );
            by_day.entry(key).or_default().push(format!(
                "- `{}` {}: {}",
                instance.availability_id,
                instance.username,
                format_interval(instance.start_utc, instance.end_utc)
            ));
        }

        for (day, entries) in by_day {
            lines.push(String::new());
            lines.push(format!("**{}**", day));
            lines.extend(entries);
        }
    } else {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for instance in instances {
            *counts
                .entry(format_interval(instance.start_utc, instance.end_utc))
                .or_insert(0) += 1;
        }

        lines.push(String::new());
        for (interval, count) in counts {
            lines.push(format!("- {}: {} persone", interval, count));
        }
    }

    reply_ephemeral(ctx, lines.join("\n")).await
}
