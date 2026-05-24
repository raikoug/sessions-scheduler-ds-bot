mod commands;
mod config;
mod db;
mod error;
mod ids;
mod models;
mod overlap;
mod permissions;
mod time;

use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use tracing::{error, info};

use crate::config::Config;
use crate::db::Db;

pub struct Data {
    pub db: Db,
    pub started_at: DateTime<Utc>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    let db = Db::connect(&config.database_url).await?;
    db.init().await?;
    let guild_id = config.guild_id;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::ss()],
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let db = db.clone();
            Box::pin(async move {
                info!("Logged in as {}", ready.user.name);
                if let Some(guild_id) = guild_id {
                    info!("Registering application commands in guild {}", guild_id);
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        serenity::GuildId::new(guild_id),
                    )
                    .await?;
                } else {
                    info!("Registering application commands globally");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                Ok(Data {
                    db,
                    started_at: Utc::now(),
                })
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged();
    let mut client = serenity::ClientBuilder::new(config.discord_token, intents)
        .framework(framework)
        .await?;

    if let Err(err) = client.start().await {
        error!(?err, "Discord client stopped with an error");
        return Err(err.into());
    }

    Ok(())
}
