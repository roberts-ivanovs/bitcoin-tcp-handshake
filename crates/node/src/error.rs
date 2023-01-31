use thiserror::Error;

use crate::connection::FromConnectionHandle;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed to send message to actor")]
    ActorSendError,
    #[error("Unexpected message from the connection: {0:?}")]
    /// The error is boxed because the FromConnectionHandle enum is very large.
    /// we want to minimise the stack footprint of the error.
    UnexpectedConnectionMessage(Box<FromConnectionHandle>),
    #[error("Connection died")]
    ActorUnavailable,
}
