use crate::*;
use thiserror::Error;

/// Error that is returned when a channel is closed.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Error)]
#[error("Channel is closed: Failed to send message {0:?}.")]
pub struct SendError<T>(pub T);

impl<T> SendError<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub(crate) fn map<T2>(self, fun: impl FnOnce(T) -> T2) -> SendError<T2> {
        SendError(fun(self.0))
    }
}

/// Error that is returned when a channel is closed or full.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Error)]
pub enum TrySendError<T> {
    #[error("Channel is closed: Failed to send message {0:?}.")]
    Closed(T),
    #[error("Channel is full: Failed to send message {0:?}.")]
    Full(T),
}

impl<T> TrySendError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::Closed(t) => t,
            Self::Full(t) => t,
        }
    }

    pub(crate) fn map<T2>(self, fun: impl FnOnce(T) -> T2) -> TrySendError<T2> {
        match self {
            Self::Closed(t) => TrySendError::Closed(fun(t)),
            Self::Full(t) => TrySendError::Full(fun(t)),
        }
    }
}

/// Error that is returned when a channel is full, or the request did nor receive a reply
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Error)]
pub enum RequestError<M, E> {
    #[error("Channel is closed: Failed to send message {0:?}.")]
    Full(M),
    #[error("No reply received: {0}")]
    NoReply(#[source] E),
}

impl<T, E> From<SendError<T>> for RequestError<T, E> {
    fn from(e: SendError<T>) -> Self {
        Self::Full(e.0)
    }
}


