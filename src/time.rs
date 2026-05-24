use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveTime, TimeZone, Timelike, Utc,
    Weekday,
};
use chrono_tz::Tz;

use crate::error::SchedulerError;
use crate::models::{AvailabilityInstance, AvailabilityRow, PlanningWeek};

pub const ROME: Tz = chrono_tz::Europe::Rome;

pub fn parse_clock(input: &str, is_end: bool) -> Result<i64, SchedulerError> {
    let normalized = input.trim().replace('.', ":");
    let parts: Vec<&str> = normalized.split(':').collect();
    if parts.len() != 2 {
        return Err(SchedulerError::InvalidInput(format!(
            "orario '{input}' non valido, usa HH:MM o HH.MM"
        )));
    }

    let hour: i64 = parts[0]
        .parse()
        .map_err(|_| SchedulerError::InvalidInput(format!("ora non valida in '{input}'")))?;
    let minute: i64 = parts[1]
        .parse()
        .map_err(|_| SchedulerError::InvalidInput(format!("minuti non validi in '{input}'")))?;

    if !(0..=59).contains(&minute) {
        return Err(SchedulerError::InvalidInput(format!(
            "minuti fuori range in '{input}'"
        )));
    }

    if hour == 24 && minute == 0 && is_end {
        return Ok(1440);
    }

    if !(0..=23).contains(&hour) {
        return Err(SchedulerError::InvalidInput(format!(
            "ora fuori range in '{input}'"
        )));
    }

    Ok(hour * 60 + minute)
}

pub fn normalize_end_minute(start_minute: i64, end_minute: i64) -> i64 {
    if end_minute <= start_minute {
        end_minute + 1440
    } else {
        end_minute
    }
}

pub fn weekday_number(day: Weekday) -> u32 {
    day.number_from_monday()
}

pub fn weekday_it(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "lunedì",
        Weekday::Tue => "martedì",
        Weekday::Wed => "mercoledì",
        Weekday::Thu => "giovedì",
        Weekday::Fri => "venerdì",
        Weekday::Sat => "sabato",
        Weekday::Sun => "domenica",
    }
}

pub fn format_time(minutes: i64) -> String {
    let m = minutes.rem_euclid(1440);
    format!("{:02}:{:02}", m / 60, m % 60)
}

pub fn format_local_date(date: NaiveDate) -> String {
    format!("{:02}/{:02}", date.day(), date.month())
}

pub fn local_datetime(
    date: NaiveDate,
    minute_of_day_or_more: i64,
) -> Result<DateTime<Tz>, SchedulerError> {
    let day_offset = minute_of_day_or_more.div_euclid(1440);
    let minute = minute_of_day_or_more.rem_euclid(1440);
    let date = date + Duration::days(day_offset);
    let time = NaiveTime::from_hms_opt((minute / 60) as u32, (minute % 60) as u32, 0)
        .ok_or_else(|| SchedulerError::InvalidInput("orario locale non valido".to_string()))?;
    let naive = date.and_time(time);

    match ROME.from_local_datetime(&naive) {
        LocalResult::Single(dt) => Ok(dt),
        LocalResult::Ambiguous(earliest, _) => Ok(earliest),
        LocalResult::None => Err(SchedulerError::InvalidInput(format!(
            "orario locale inesistente per cambio ora: {naive}"
        ))),
    }
}

pub fn next_local_start(
    day: Weekday,
    start_minute: i64,
    now_utc: DateTime<Utc>,
) -> Result<DateTime<Tz>, SchedulerError> {
    let now = now_utc.with_timezone(&ROME);
    let today = now.date_naive();
    let now_minutes = now.hour() as i64 * 60 + now.minute() as i64;

    let current = now.weekday().number_from_monday() as i64;
    let target = day.number_from_monday() as i64;
    let mut days_ahead = target - current;

    if days_ahead < 0 || (days_ahead == 0 && start_minute <= now_minutes) {
        days_ahead += 7;
    }

    let start_date = today + Duration::days(days_ahead);
    local_datetime(start_date, start_minute)
}

pub fn concrete_interval_for_next_occurrence(
    day: Weekday,
    start_minute: i64,
    end_minute: i64,
    now_utc: DateTime<Utc>,
) -> Result<(DateTime<Utc>, DateTime<Utc>, i64), SchedulerError> {
    let normalized_end = normalize_end_minute(start_minute, end_minute);
    let start = next_local_start(day, start_minute, now_utc)?;
    let end = local_datetime(start.date_naive(), normalized_end)?;
    Ok((
        start.with_timezone(&Utc),
        end.with_timezone(&Utc),
        normalized_end,
    ))
}

/// Planning window for `/ss week` and `/ss overlaps`:
/// always the next Monday-Sunday week in Europe/Rome.
pub fn planning_week(now_utc: DateTime<Utc>) -> Result<PlanningWeek, SchedulerError> {
    let now = now_utc.with_timezone(&ROME);
    let today = now.date_naive();
    let current_monday = today - Duration::days((now.weekday().number_from_monday() - 1) as i64);
    let start_date = current_monday + Duration::days(7);
    let end_date = start_date + Duration::days(7);
    let start = local_datetime(start_date, 0)?;
    let end = local_datetime(end_date, 0)?;

    Ok(PlanningWeek {
        start_date,
        end_date,
        start_utc: start.with_timezone(&Utc),
        end_utc: end.with_timezone(&Utc),
    })
}

pub fn instance_for_week(
    row: &AvailabilityRow,
    week: PlanningWeek,
) -> Result<Option<AvailabilityInstance>, SchedulerError> {
    let start_utc = if row.is_recurring {
        let day_offset = row.weekday as i64 - 1;
        let date = week.start_date + Duration::days(day_offset);
        local_datetime(date, row.start_minute)?.with_timezone(&Utc)
    } else {
        row.starts_at_utc
    };

    let end_utc = if row.is_recurring {
        let day_offset = row.weekday as i64 - 1;
        let date = week.start_date + Duration::days(day_offset);
        local_datetime(date, row.end_minute)?.with_timezone(&Utc)
    } else {
        row.ends_at_utc
    };

    if end_utc <= week.start_utc || start_utc >= week.end_utc {
        return Ok(None);
    }

    Ok(Some(AvailabilityInstance {
        availability_id: row.id.clone(),
        user_id: row.user_id.clone(),
        username: row.username.clone(),
        start_utc,
        end_utc,
    }))
}

pub fn format_availability_row(row: &AvailabilityRow) -> String {
    let recurring = if row.is_recurring {
        "ricorrente"
    } else {
        "non ricorrente"
    };
    if row.is_recurring {
        format!(
            "{}: {} dalle {} alle {} {}",
            row.id,
            weekday_it(row.weekday_enum()),
            format_time(row.start_minute),
            format_time(row.end_minute),
            recurring
        )
    } else {
        let start = row.starts_at_utc.with_timezone(&ROME);
        let end = row.ends_at_utc.with_timezone(&ROME);
        format!(
            "{}: {} {} dalle {} fino a {} {} alle {} {}",
            row.id,
            weekday_it(start.weekday()),
            format_local_date(start.date_naive()),
            start.format("%H:%M"),
            weekday_it(end.weekday()),
            format_local_date(end.date_naive()),
            end.format("%H:%M"),
            recurring
        )
    }
}

pub fn format_interval(start_utc: DateTime<Utc>, end_utc: DateTime<Utc>) -> String {
    let start = start_utc.with_timezone(&ROME);
    let end = end_utc.with_timezone(&ROME);
    if start.date_naive() == end.date_naive() {
        format!(
            "{} {} dalle {} alle {}",
            weekday_it(start.weekday()),
            format_local_date(start.date_naive()),
            start.format("%H:%M"),
            end.format("%H:%M")
        )
    } else {
        format!(
            "{} {} dalle {} fino a {} {} alle {}",
            weekday_it(start.weekday()),
            format_local_date(start.date_naive()),
            start.format("%H:%M"),
            weekday_it(end.weekday()),
            format_local_date(end.date_naive()),
            end.format("%H:%M")
        )
    }
}

pub fn iso_utc(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

pub fn parse_utc(s: &str) -> Result<DateTime<Utc>, SchedulerError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| SchedulerError::InvalidInput(format!("datetime UTC non valido nel DB: {e}")))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc, Weekday};

    use super::*;

    #[test]
    fn normalizes_overnight_end_minutes() {
        assert_eq!(normalize_end_minute(21 * 60, 2 * 60), 26 * 60);
    }

    #[test]
    fn accepts_24_00_only_for_end() {
        assert_eq!(parse_clock("24:00", true).unwrap(), 1440);
        assert!(parse_clock("24:00", false).is_err());
    }

    #[test]
    fn computes_next_non_recurring_overnight_interval() {
        let now = Utc.with_ymd_and_hms(2026, 5, 25, 0, 0, 0).unwrap();
        let (start, end, normalized_end) =
            concrete_interval_for_next_occurrence(Weekday::Mon, 21 * 60, 2 * 60, now).unwrap();

        assert_eq!(normalized_end, 26 * 60);
        assert_eq!(
            start
                .with_timezone(&ROME)
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            "2026-05-25 21:00"
        );
        assert_eq!(
            end.with_timezone(&ROME)
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            "2026-05-26 02:00"
        );
    }

    #[test]
    fn planning_week_targets_next_monday_to_sunday() {
        let monday = Utc.with_ymd_and_hms(2026, 5, 25, 10, 0, 0).unwrap();
        let sunday = Utc.with_ymd_and_hms(2026, 5, 31, 10, 0, 0).unwrap();

        let monday_week = planning_week(monday).unwrap();
        let sunday_week = planning_week(sunday).unwrap();

        assert_eq!(
            monday_week.start_date,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
        );
        assert_eq!(
            sunday_week.start_date,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
        );
        assert_eq!(
            monday_week.end_date,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap()
        );
    }
}
