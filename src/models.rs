use chrono::{DateTime, NaiveDate, Utc, Weekday};

#[derive(Debug, Clone)]
pub struct AvailabilityRow {
    pub id: String,
    pub guild_id: String,
    pub user_id: String,
    pub username: String,
    pub weekday: u32,
    pub start_minute: i64,
    pub end_minute: i64,
    pub starts_at_utc: DateTime<Utc>,
    pub ends_at_utc: DateTime<Utc>,
    pub is_recurring: bool,
}

impl AvailabilityRow {
    pub fn weekday_enum(&self) -> Weekday {
        match self.weekday {
            1 => Weekday::Mon,
            2 => Weekday::Tue,
            3 => Weekday::Wed,
            4 => Weekday::Thu,
            5 => Weekday::Fri,
            6 => Weekday::Sat,
            _ => Weekday::Sun,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewAvailability {
    pub id: String,
    pub guild_id: String,
    pub user_id: String,
    pub username: String,
    pub weekday: u32,
    pub start_minute: i64,
    pub end_minute: i64,
    pub starts_at_utc: DateTime<Utc>,
    pub ends_at_utc: DateTime<Utc>,
    pub is_recurring: bool,
}

#[derive(Debug, Clone)]
pub struct AvailabilityInstance {
    pub availability_id: String,
    pub user_id: String,
    pub username: String,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OverlapSlot {
    pub display_index: i64,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
    pub participant_user_ids: Vec<String>,
    pub participant_usernames: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StoredOverlapItem {
    pub id: String,
    pub display_index: i64,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
    pub start_local: String,
    pub end_local: String,
    pub participant_user_ids: Vec<String>,
    pub participant_usernames: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScheduledSession {
    pub id: String,
    pub guild_id: String,
    pub scheduled_by_user_id: String,
    pub scheduled_by_username: String,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
    pub start_local: String,
    pub end_local: String,
    pub participant_user_ids: Vec<String>,
    pub participant_usernames: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlanningWeek {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
}
