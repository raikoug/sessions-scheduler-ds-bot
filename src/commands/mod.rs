#[path = "config.rs"]
pub mod config_cmd;
#[path = "help.rs"]
pub mod help_cmd;
#[path = "ican.rs"]
pub mod ican_cmd;
#[path = "list.rs"]
pub mod list_cmd;
#[path = "overlaps.rs"]
pub mod overlaps_cmd;
#[path = "remove.rs"]
pub mod remove_cmd;
#[path = "schedule.rs"]
pub mod schedule_cmd;
#[path = "week.rs"]
pub mod week_cmd;

pub use config_cmd::config;
pub use help_cmd::help;
pub use ican_cmd::ican;
pub use list_cmd::list;
pub use overlaps_cmd::overlaps;
pub use remove_cmd::remove;
pub use schedule_cmd::schedule;
pub use week_cmd::week;

use chrono::{DateTime, Utc, Weekday};
use poise::CreateReply;

use crate::error::SchedulerError;
use crate::models::{AvailabilityInstance, PlanningWeek};
use crate::time::{instance_for_week, planning_week};
use crate::{Context, Error};

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum Giorno {
    #[name = "lun"]
    Lun,
    #[name = "mar"]
    Mar,
    #[name = "merc"]
    Merc,
    #[name = "gio"]
    Gio,
    #[name = "ven"]
    Ven,
    #[name = "sab"]
    Sab,
    #[name = "dom"]
    Dom,
}

impl Giorno {
    pub fn weekday(self) -> Weekday {
        match self {
            Self::Lun => Weekday::Mon,
            Self::Mar => Weekday::Tue,
            Self::Merc => Weekday::Wed,
            Self::Gio => Weekday::Thu,
            Self::Ven => Weekday::Fri,
            Self::Sab => Weekday::Sat,
            Self::Dom => Weekday::Sun,
        }
    }
}

#[poise::command(
    slash_command,
    subcommands(
        "help", "ican", "list", "remove", "week", "overlaps", "schedule", "config"
    )
)]
/// Comandi del bot per disponibilita', overlap e sessioni.
pub async fn ss(ctx: Context<'_>) -> Result<(), Error> {
    reply_ephemeral(
        ctx,
        "Usa un sottocomando: `/ss help`, `/ss ican`, `/ss list`, `/ss remove`, `/ss week`, `/ss overlaps`, `/ss schedule`.",
    )
    .await
}

pub fn guild_id_string(ctx: Context<'_>) -> Result<String, SchedulerError> {
    ctx.guild_id()
        .map(|id| id.to_string())
        .ok_or(SchedulerError::MissingGuild)
}

pub fn author_id_string(ctx: Context<'_>) -> String {
    ctx.author().id.to_string()
}

pub fn author_name(ctx: Context<'_>) -> String {
    ctx.author()
        .global_name
        .clone()
        .unwrap_or_else(|| ctx.author().name.clone())
}

pub async fn reply_ephemeral(ctx: Context<'_>, content: impl Into<String>) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .content(content.into())
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

pub async fn planning_instances(
    ctx: Context<'_>,
    now: DateTime<Utc>,
) -> Result<(PlanningWeek, Vec<AvailabilityInstance>), Error> {
    let guild_id = guild_id_string(ctx)?;
    let week = planning_week(now)?;
    let rows = ctx.data().db.list_active_availabilities(&guild_id).await?;

    let mut instances = Vec::new();
    for row in rows {
        if let Some(instance) = instance_for_week(&row, week)? {
            if instance.end_utc > now {
                instances.push(instance);
            }
        }
    }

    instances.sort_by_key(|i| (i.start_utc, i.end_utc, i.user_id.clone()));
    Ok((week, instances))
}
