PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS guild_settings (
    guild_id TEXT PRIMARY KEY,
    timezone TEXT NOT NULL DEFAULT 'Europe/Rome',
    schedule_channel_id TEXT,
    created_at_utc TEXT NOT NULL,
    updated_at_utc TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS availabilities (
    id TEXT PRIMARY KEY,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT NOT NULL,
    weekday INTEGER NOT NULL CHECK (weekday BETWEEN 1 AND 7),
    start_minute INTEGER NOT NULL CHECK (start_minute BETWEEN 0 AND 1439),
    end_minute INTEGER NOT NULL CHECK (end_minute BETWEEN 1 AND 2880),
    starts_at_utc TEXT NOT NULL,
    ends_at_utc TEXT NOT NULL,
    is_recurring INTEGER NOT NULL CHECK (is_recurring IN (0, 1)),
    created_at_utc TEXT NOT NULL,
    removed_at_utc TEXT
);

CREATE INDEX IF NOT EXISTS idx_availabilities_guild_user_active
ON availabilities (guild_id, user_id, removed_at_utc);

CREATE INDEX IF NOT EXISTS idx_availabilities_guild_active
ON availabilities (guild_id, removed_at_utc);

CREATE TABLE IF NOT EXISTS overlap_batches (
    id TEXT PRIMARY KEY,
    guild_id TEXT NOT NULL,
    week_start_date TEXT NOT NULL,
    created_by_user_id TEXT NOT NULL,
    created_by_username TEXT NOT NULL,
    created_at_utc TEXT NOT NULL,
    is_valid INTEGER NOT NULL CHECK (is_valid IN (0, 1)) DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_overlap_batches_guild_valid
ON overlap_batches (guild_id, is_valid, created_at_utc);

CREATE TABLE IF NOT EXISTS overlap_items (
    id TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL,
    display_index INTEGER NOT NULL,
    day_label TEXT NOT NULL,
    start_utc TEXT NOT NULL,
    end_utc TEXT NOT NULL,
    start_local TEXT NOT NULL,
    end_local TEXT NOT NULL,
    participant_user_ids TEXT NOT NULL,
    participant_usernames TEXT NOT NULL,
    FOREIGN KEY (batch_id) REFERENCES overlap_batches(id) ON DELETE CASCADE,
    UNIQUE (batch_id, display_index)
);

CREATE TABLE IF NOT EXISTS scheduled_sessions (
    id TEXT PRIMARY KEY,
    guild_id TEXT NOT NULL,
    overlap_item_id TEXT NOT NULL,
    scheduled_by_user_id TEXT NOT NULL,
    scheduled_by_username TEXT NOT NULL,
    scheduled_at_utc TEXT NOT NULL,
    start_utc TEXT NOT NULL,
    end_utc TEXT NOT NULL,
    start_local TEXT NOT NULL,
    end_local TEXT NOT NULL,
    participant_user_ids TEXT NOT NULL,
    participant_usernames TEXT NOT NULL,
    channel_id TEXT,
    message_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_scheduled_sessions_guild_future
ON scheduled_sessions (guild_id, start_utc);
