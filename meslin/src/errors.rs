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

    pub fn map<T2>(self, fun: impl FnOnce(T) -> T2) -> SendError<T2> {
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

/// Error that is returned when a channel is closed, or the message was not accepted.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Error)]
pub enum DynSendError<T> {
    #[error("Message {0:?} was not accepted.")]
    NotAccepted(T),
    #[error("Channel is closed: Failed to send message {0:?}.")]
    Closed(T),
}

impl<T> DynSendError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::NotAccepted(t) => t,
            Self::Closed(t) => t,
        }
    }

    pub(crate) fn map<U>(self, f: impl FnOnce(T) -> U) -> DynSendError<U> {
        match self {
            Self::NotAccepted(t) => DynSendError::NotAccepted(f(t)),
            Self::Closed(t) => DynSendError::Closed(f(t)),
        }
    }
}

impl<W: 'static> DynSendError<BoxedMsg<W>> {
    pub(crate) fn downcast<M: 'static>(self) -> Result<DynSendError<(M, W)>, Self> {
        match self {
            Self::NotAccepted(t) => match t.downcast::<M>() {
                Ok(t) => Ok(DynSendError::NotAccepted(t)),
                Err(t) => Err(DynSendError::NotAccepted(t)),
            },
            Self::Closed(t) => match t.downcast::<M>() {
                Ok(t) => Ok(DynSendError::Closed(t)),
                Err(t) => Err(DynSendError::Closed(t)),
            },
        }
    }
}

impl<T> From<SendError<T>> for DynSendError<T> {
    fn from(SendError(t): SendError<T>) -> Self {
        Self::Closed(t)
    }
}

/// Error that is returned when a channel is closed, full, or the message was not accepted.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Error)]
pub enum DynTrySendError<T> {
    #[error("Message {0:?} was not accepted.")]
    NotAccepted(T),
    #[error("Channel is closed: Failed to send message {0:?}.")]
    Closed(T),
    #[error("Channel is full: Failed to send message {0:?}.")]
    Full(T),
}

impl<T> DynTrySendError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::NotAccepted(t) => t,
            Self::Closed(t) => t,
            Self::Full(t) => t,
        }
    }

    pub(crate) fn map<U>(self, f: impl FnOnce(T) -> U) -> DynTrySendError<U> {
        match self {
            Self::NotAccepted(t) => DynTrySendError::NotAccepted(f(t)),
            Self::Closed(t) => DynTrySendError::Closed(f(t)),
            Self::Full(t) => DynTrySendError::Full(f(t)),
        }
    }
}

impl<W: 'static> DynTrySendError<BoxedMsg<W>> {
    pub(crate) fn downcast<M: 'static>(self) -> Result<DynTrySendError<(M, W)>, Self> {
        match self {
            Self::NotAccepted(t) => match t.downcast::<M>() {
                Ok(t) => Ok(DynTrySendError::NotAccepted(t)),
                Err(t) => Err(DynTrySendError::NotAccepted(t)),
            },
            Self::Closed(t) => match t.downcast::<M>() {
                Ok(t) => Ok(DynTrySendError::Closed(t)),
                Err(t) => Err(DynTrySendError::Closed(t)),
            },
            Self::Full(t) => match t.downcast::<M>() {
                Ok(t) => Ok(DynTrySendError::Full(t)),
                Err(t) => Err(DynTrySendError::Full(t)),
            },
        }
    }
}

impl<T> From<SendError<T>> for DynTrySendError<T> {
    fn from(SendError(t): SendError<T>) -> Self {
        Self::Closed(t)
    }
}

impl<T> From<TrySendError<T>> for DynTrySendError<T> {
    fn from(e: TrySendError<T>) -> Self {
        match e {
            TrySendError::Closed(t) => Self::Closed(t),
            TrySendError::Full(t) => Self::Full(t),
        }
    }
}
