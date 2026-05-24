use crate::{Context, Error};

use super::reply_ephemeral;

#[poise::command(slash_command)]
/// Mostra una guida rapida per usare SessionScheduler.
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let message = "\
**SessionScheduler: guida rapida**\n\
\n\
`/ss ican` aggiunge una tua disponibilita'. Usa `giorno`, `start`, `end` e facoltativamente `ricorrente:true`.\n\
`/ss list` mostra solo le tue disponibilita' future con il loro ID.\n\
`/ss remove` rimuove una disponibilita' usando il suo ID.\n\
`/ss week` mostra le disponibilita' della prossima settimana.\n\
`/ss overlaps` cerca gli slot della prossima settimana in cui tutti gli utenti attivi sono disponibili.\n\
`/ss schedule` permette a un admin di fissare una sessione da un overlap ID gia' salvato.\n\
`/ss config channel` permette a un admin di scegliere il canale dove pubblicare le sessioni.\n\
\n\
Esempio rapido:\n\
`/ss ican giorno:lun start:21:00 end:02:00`\n\
\n\
Note:\n\
- accetta sia `21:00` sia `21.00`\n\
- se `end <= start`, la disponibilita' passa a dopo mezzanotte\n\
- per pianificare una sessione, un admin deve prima eseguire `/ss overlaps`";

    reply_ephemeral(ctx, message).await
}
