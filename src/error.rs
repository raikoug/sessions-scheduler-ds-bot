use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("questo comando deve essere usato dentro un server Discord")]
    MissingGuild,

    #[error("input non valido: {0}")]
    InvalidInput(String),

    #[error("nessun canale sessioni configurato: usa prima /ss config channel")]
    MissingScheduleChannel,

    #[error("operazione non permessa")]
    Forbidden,

    #[error("risorsa non trovata")]
    NotFound,

    #[error("nessuna tabella overlap valida: esegui prima /ss overlaps come admin")]
    MissingOverlapBatch,
}
