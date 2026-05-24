use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
    pub guild_id: Option<u64>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let discord_token = env::var("DISCORD_TOKEN")
            .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN is missing from environment"))?;
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://session_scheduler.sqlite".to_string());
        let guild_id = env::var("GUILD_ID")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("GUILD_ID must be a valid Discord guild id"))
            })
            .transpose()?;

        Ok(Self {
            discord_token,
            database_url,
            guild_id,
        })
    }
}
