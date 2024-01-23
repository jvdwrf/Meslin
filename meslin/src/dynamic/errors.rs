use crate::*;
use thiserror::Error;

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
pub enum DynSendNowError<T> {
    #[error("Message {0:?} was not accepted.")]
    NotAccepted(T),
    #[error("Channel is closed: Failed to send message {0:?}.")]
    Closed(T),
    #[error("Channel is full: Failed to send message {0:?}.")]
    Full(T),
}

impl<T> DynSendNowError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::NotAccepted(t) => t,
            Self::Closed(t) => t,
            Self::Full(t) => t,
        }
    }

    pub(crate) fn map<U>(self, f: impl FnOnce(T) -> U) -> DynSendNowError<U> {
        match self {
            Self::NotAccepted(t) => DynSendNowError::NotAccepted(f(t)),
            Self::Closed(t) => DynSendNowError::Closed(f(t)),
            Self::Full(t) => DynSendNowError::Full(f(t)),
        }
    }
}

impl<W: 'static> DynSendNowError<BoxedMsg<W>> {
    pub(crate) fn downcast<M: 'static>(self) -> Result<DynSendNowError<(M, W)>, Self> {
        match self {
            Self::NotAccepted(t) => match t.downcast::<M>() {
                Ok(t) => Ok(DynSendNowError::NotAccepted(t)),
                Err(t) => Err(DynSendNowError::NotAccepted(t)),
            },
            Self::Closed(t) => match t.downcast::<M>() {
                Ok(t) => Ok(DynSendNowError::Closed(t)),
                Err(t) => Err(DynSendNowError::Closed(t)),
            },
            Self::Full(t) => match t.downcast::<M>() {
                Ok(t) => Ok(DynSendNowError::Full(t)),
                Err(t) => Err(DynSendNowError::Full(t)),
            },
        }
    }
}

impl<T> From<SendError<T>> for DynSendNowError<T> {
    fn from(SendError(t): SendError<T>) -> Self {
        Self::Closed(t)
    }
}

impl<T> From<SendNowError<T>> for DynSendNowError<T> {
    fn from(e: SendNowError<T>) -> Self {
        match e {
            SendNowError::Closed(t) => Self::Closed(t),
            SendNowError::Full(t) => Self::Full(t),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Error)]
pub enum DynRequestError<M, E> {
    #[error("Message {0:?} was not accepted.")]
    NotAccepted(M),
    #[error("Channel is closed: Failed to send message.")]
    Full(M),
    #[error("No reply received: {0}")]
    NoReply(#[source] E),
}

impl<M, E> From<DynSendError<M>> for DynRequestError<M, E> {
    fn from(e: DynSendError<M>) -> Self {
        match e {
            DynSendError::NotAccepted(m) => Self::NotAccepted(m),
            DynSendError::Closed(m) => Self::Full(m),
        }
    }
}
