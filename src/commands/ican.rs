use chrono::Utc;

use crate::ids::short_id;
use crate::models::NewAvailability;
use crate::time::{
    concrete_interval_for_next_occurrence, format_interval, format_time, normalize_end_minute,
    parse_clock, weekday_it, weekday_number,
};
use crate::{Context, Error};

use super::{author_id_string, author_name, guild_id_string, reply_ephemeral, Giorno};

#[poise::command(slash_command)]
/// Aggiunge una disponibilita' tua per un giorno e orario.
pub async fn ican(
    ctx: Context<'_>,
    #[description = "Giorno: lun, mar, merc, gio, ven, sab, dom"] giorno: Giorno,
    #[description = "Ora inizio, per esempio 21:00 o 21.00"] start: String,
    #[description = "Ora fine, per esempio 23:30, 02:00 o 24:00"] end: String,
    #[description = "Se true, si ripete ogni settimana. Default: false"] ricorrente: Option<bool>,
) -> Result<(), Error> {
    let guild_id = guild_id_string(ctx)?;
    let user_id = author_id_string(ctx);
    let username = author_name(ctx);
    let now = Utc::now();
    let day = giorno.weekday();
    let is_recurring = ricorrente.unwrap_or(false);

    let start_minute = parse_clock(&start, false)?;
    let raw_end_minute = parse_clock(&end, true)?;
    let end_minute = normalize_end_minute(start_minute, raw_end_minute);
    let (starts_at_utc, ends_at_utc, normalized_end) =
        concrete_interval_for_next_occurrence(day, start_minute, raw_end_minute, now)?;

    let id = short_id();
    let availability = NewAvailability {
        id: id.clone(),
        guild_id,
        user_id,
        username,
        weekday: weekday_number(day),
        start_minute,
        end_minute: normalized_end,
        starts_at_utc,
        ends_at_utc,
        is_recurring,
    };

    ctx.data().db.insert_availability(&availability).await?;

    let message = if is_recurring {
        format!(
            "Disponibilità aggiunta: `{}`: {} dalle {} alle {} ricorrente.",
            id,
            weekday_it(day),
            format_time(start_minute),
            format_time(end_minute)
        )
    } else {
        format!(
            "Disponibilità aggiunta: `{}`. Hai messo disponibilità {} non ricorrente.",
            id,
            format_interval(starts_at_utc, ends_at_utc)
        )
    };

    reply_ephemeral(ctx, message).await
}
