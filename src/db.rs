use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use crate::error::SchedulerError;
use crate::ids::short_id;
use crate::models::{
    AvailabilityRow, NewAvailability, OverlapSlot, ScheduledSession, StoredOverlapItem,
};
use crate::time::{format_interval, iso_utc, parse_utc};

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        Ok(Self { pool })
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        let schema = include_str!("../migrations/0001_init.sql");
        for statement in schema.split(';') {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }
            sqlx::query(statement).execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn ensure_guild_settings(&self, guild_id: &str) -> anyhow::Result<()> {
        let now = iso_utc(Utc::now());
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO guild_settings (guild_id, timezone, created_at_utc, updated_at_utc)
            VALUES (?1, 'Europe/Rome', ?2, ?2)
            "#,
        )
        .bind(guild_id)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_schedule_channel(
        &self,
        guild_id: &str,
        channel_id: &str,
    ) -> anyhow::Result<()> {
        self.ensure_guild_settings(guild_id).await?;
        let now = iso_utc(Utc::now());
        sqlx::query(
            r#"
            UPDATE guild_settings
            SET schedule_channel_id = ?2, updated_at_utc = ?3
            WHERE guild_id = ?1
            "#,
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn schedule_channel(&self, guild_id: &str) -> anyhow::Result<Option<String>> {
        self.ensure_guild_settings(guild_id).await?;
        let row = sqlx::query("SELECT schedule_channel_id FROM guild_settings WHERE guild_id = ?1")
            .bind(guild_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.and_then(|r| {
            r.try_get::<Option<String>, _>("schedule_channel_id")
                .ok()
                .flatten()
        }))
    }

    pub async fn insert_availability(&self, availability: &NewAvailability) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO availabilities (
                id, guild_id, user_id, username, weekday, start_minute, end_minute,
                starts_at_utc, ends_at_utc, is_recurring, created_at_utc
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&availability.id)
        .bind(&availability.guild_id)
        .bind(&availability.user_id)
        .bind(&availability.username)
        .bind(availability.weekday as i64)
        .bind(availability.start_minute)
        .bind(availability.end_minute)
        .bind(iso_utc(availability.starts_at_utc))
        .bind(iso_utc(availability.ends_at_utc))
        .bind(if availability.is_recurring { 1 } else { 0 })
        .bind(iso_utc(Utc::now()))
        .execute(&self.pool)
        .await?;

        self.invalidate_overlap_batches(&availability.guild_id)
            .await?;
        Ok(())
    }

    pub async fn list_user_future_availabilities(
        &self,
        guild_id: &str,
        user_id: &str,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<AvailabilityRow>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM availabilities
            WHERE guild_id = ?1
              AND user_id = ?2
              AND removed_at_utc IS NULL
              AND (is_recurring = 1 OR ends_at_utc > ?3)
            ORDER BY weekday, start_minute
            "#,
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(iso_utc(now))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_availability).collect()
    }

    pub async fn list_active_availabilities(
        &self,
        guild_id: &str,
    ) -> anyhow::Result<Vec<AvailabilityRow>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM availabilities
            WHERE guild_id = ?1
              AND removed_at_utc IS NULL
            ORDER BY user_id, weekday, start_minute
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_availability).collect()
    }

    pub async fn find_active_availability(
        &self,
        guild_id: &str,
        availability_id: &str,
    ) -> anyhow::Result<Option<AvailabilityRow>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM availabilities
            WHERE guild_id = ?1 AND id = ?2 AND removed_at_utc IS NULL
            "#,
        )
        .bind(guild_id)
        .bind(availability_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_availability).transpose()
    }

    pub async fn soft_remove_availability(
        &self,
        guild_id: &str,
        availability_id: &str,
    ) -> anyhow::Result<()> {
        let now = iso_utc(Utc::now());
        sqlx::query(
            r#"
            UPDATE availabilities
            SET removed_at_utc = ?3
            WHERE guild_id = ?1 AND id = ?2 AND removed_at_utc IS NULL
            "#,
        )
        .bind(guild_id)
        .bind(availability_id)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.invalidate_overlap_batches(guild_id).await?;
        Ok(())
    }

    pub async fn invalidate_overlap_batches(&self, guild_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE overlap_batches
            SET is_valid = 0
            WHERE guild_id = ?1 AND is_valid = 1
            "#,
        )
        .bind(guild_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn store_overlap_batch(
        &self,
        guild_id: &str,
        week_start_date: &str,
        created_by_user_id: &str,
        created_by_username: &str,
        slots: &[OverlapSlot],
    ) -> anyhow::Result<String> {
        self.invalidate_overlap_batches(guild_id).await?;
        let batch_id = short_id();
        let now = iso_utc(Utc::now());

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO overlap_batches (
                id, guild_id, week_start_date, created_by_user_id, created_by_username, created_at_utc, is_valid
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)
            "#,
        )
        .bind(&batch_id)
        .bind(guild_id)
        .bind(week_start_date)
        .bind(created_by_user_id)
        .bind(created_by_username)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        for slot in slots {
            let item_id = short_id();
            let interval = format_interval(slot.start_utc, slot.end_utc);
            let user_ids = slot.participant_user_ids.join(",");
            let usernames = slot.participant_usernames.join(", ");
            sqlx::query(
                r#"
                INSERT INTO overlap_items (
                    id, batch_id, display_index, day_label, start_utc, end_utc,
                    start_local, end_local, participant_user_ids, participant_usernames
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#,
            )
            .bind(item_id)
            .bind(&batch_id)
            .bind(slot.display_index)
            .bind(&interval)
            .bind(iso_utc(slot.start_utc))
            .bind(iso_utc(slot.end_utc))
            .bind(&interval)
            .bind(&interval)
            .bind(user_ids)
            .bind(usernames)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(batch_id)
    }

    pub async fn latest_valid_overlap_item(
        &self,
        guild_id: &str,
        display_index: i64,
    ) -> anyhow::Result<Option<StoredOverlapItem>> {
        let row = sqlx::query(
            r#"
            SELECT oi.*
            FROM overlap_items oi
            JOIN overlap_batches ob ON ob.id = oi.batch_id
            WHERE ob.guild_id = ?1 AND ob.is_valid = 1 AND oi.display_index = ?2
            ORDER BY ob.created_at_utc DESC
            LIMIT 1
            "#,
        )
        .bind(guild_id)
        .bind(display_index)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_overlap_item).transpose()
    }

    pub async fn insert_scheduled_session(
        &self,
        guild_id: &str,
        overlap_item_id: &str,
        scheduled_by: (&str, &str),
        item: &StoredOverlapItem,
        message_ref: Option<(&str, &str)>,
    ) -> anyhow::Result<String> {
        let id = short_id();
        let (scheduled_by_user_id, scheduled_by_username) = scheduled_by;
        let (channel_id, message_id) = match message_ref {
            Some((channel_id, message_id)) => (Some(channel_id), Some(message_id)),
            None => (None, None),
        };
        sqlx::query(
            r#"
            INSERT INTO scheduled_sessions (
                id, guild_id, overlap_item_id, scheduled_by_user_id, scheduled_by_username,
                scheduled_at_utc, start_utc, end_utc, start_local, end_local,
                participant_user_ids, participant_usernames, channel_id, message_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
        )
        .bind(&id)
        .bind(guild_id)
        .bind(overlap_item_id)
        .bind(scheduled_by_user_id)
        .bind(scheduled_by_username)
        .bind(iso_utc(Utc::now()))
        .bind(iso_utc(item.start_utc))
        .bind(iso_utc(item.end_utc))
        .bind(&item.start_local)
        .bind(&item.end_local)
        .bind(item.participant_user_ids.join(","))
        .bind(item.participant_usernames.join(", "))
        .bind(channel_id)
        .bind(message_id)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn scheduled_conflicts_for_removed_availability(
        &self,
        guild_id: &str,
        user_id: &str,
        start_utc: DateTime<Utc>,
        end_utc: DateTime<Utc>,
    ) -> anyhow::Result<Vec<ScheduledSession>> {
        let now = iso_utc(Utc::now());
        let rows = sqlx::query(
            r#"
            SELECT * FROM scheduled_sessions
            WHERE guild_id = ?1
              AND start_utc > ?2
              AND participant_user_ids LIKE ?3
              AND start_utc < ?5
              AND end_utc > ?4
            ORDER BY start_utc
            "#,
        )
        .bind(guild_id)
        .bind(now)
        .bind(format!("%{}%", user_id))
        .bind(iso_utc(start_utc))
        .bind(iso_utc(end_utc))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_scheduled_session).collect()
    }
}

fn row_to_availability(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<AvailabilityRow> {
    let starts_at = row.try_get::<String, _>("starts_at_utc")?;
    let ends_at = row.try_get::<String, _>("ends_at_utc")?;
    Ok(AvailabilityRow {
        id: row.try_get("id")?,
        guild_id: row.try_get("guild_id")?,
        user_id: row.try_get("user_id")?,
        username: row.try_get("username")?,
        weekday: row.try_get::<i64, _>("weekday")? as u32,
        start_minute: row.try_get("start_minute")?,
        end_minute: row.try_get("end_minute")?,
        starts_at_utc: parse_utc(&starts_at)?,
        ends_at_utc: parse_utc(&ends_at)?,
        is_recurring: row.try_get::<i64, _>("is_recurring")? == 1,
    })
}

fn row_to_overlap_item(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<StoredOverlapItem> {
    let start_utc = row.try_get::<String, _>("start_utc")?;
    let end_utc = row.try_get::<String, _>("end_utc")?;
    let user_ids = split_csv(row.try_get::<String, _>("participant_user_ids")?);
    let usernames = split_csv(row.try_get::<String, _>("participant_usernames")?);
    Ok(StoredOverlapItem {
        id: row.try_get("id")?,
        display_index: row.try_get("display_index")?,
        start_utc: parse_utc(&start_utc)?,
        end_utc: parse_utc(&end_utc)?,
        start_local: row.try_get("start_local")?,
        end_local: row.try_get("end_local")?,
        participant_user_ids: user_ids,
        participant_usernames: usernames,
    })
}

fn row_to_scheduled_session(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<ScheduledSession> {
    let start_utc = row.try_get::<String, _>("start_utc")?;
    let end_utc = row.try_get::<String, _>("end_utc")?;
    Ok(ScheduledSession {
        id: row.try_get("id")?,
        guild_id: row.try_get("guild_id")?,
        scheduled_by_user_id: row.try_get("scheduled_by_user_id")?,
        scheduled_by_username: row.try_get("scheduled_by_username")?,
        start_utc: parse_utc(&start_utc)?,
        end_utc: parse_utc(&end_utc)?,
        start_local: row.try_get("start_local")?,
        end_local: row.try_get("end_local")?,
        participant_user_ids: split_csv(row.try_get::<String, _>("participant_user_ids")?),
        participant_usernames: split_csv(row.try_get::<String, _>("participant_usernames")?),
    })
}

fn split_csv(value: String) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn assert_found<T>(value: Option<T>) -> Result<T, SchedulerError> {
    value.ok_or(SchedulerError::NotFound)
}
