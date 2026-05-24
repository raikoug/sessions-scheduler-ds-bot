use poise::serenity_prelude as serenity;

use crate::permissions::{is_admin, require_admin_sync};
use crate::{Context, Error};

use super::{guild_id_string, reply_ephemeral};

#[poise::command(slash_command, subcommands("channel"))]
/// Configurazione amministrativa del bot.
pub async fn config(ctx: Context<'_>) -> Result<(), Error> {
    reply_ephemeral(ctx, "Configura il bot con `/ss config channel`.").await
}

#[poise::command(slash_command)]
/// Imposta il canale dove pubblicare le sessioni schedulate.
pub async fn channel(
    ctx: Context<'_>,
    #[description = "Canale dove pubblicare le sessioni schedulate"] channel: serenity::ChannelId,
) -> Result<(), Error> {
    let admin = is_admin(ctx).await?;
    require_admin_sync(admin)?;

    let guild_id = guild_id_string(ctx)?;
    ctx.data()
        .db
        .set_schedule_channel(&guild_id, &channel.to_string())
        .await?;

    reply_ephemeral(ctx, format!("Canale sessioni configurato: <#{}>", channel)).await
}
