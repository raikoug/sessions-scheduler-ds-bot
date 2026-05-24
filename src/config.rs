use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let discord_token = env::var("DISCORD_TOKEN")
            .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN is missing from environment"))?;
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://session_scheduler.sqlite".to_string());

        Ok(Self {
            discord_token,
            database_url,
        })
    }
}
