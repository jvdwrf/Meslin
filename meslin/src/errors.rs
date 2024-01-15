use crate::*;
use thiserror::Error;

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

impl<T, W> SendError<(T, W)> {
    pub(crate) fn map_first(self) -> SendError<T> {
        SendError(self.0 .0)
    }

    pub(crate) fn map_into_msg_unwrap<M2>(self) -> SendError<(M2, W)>
    where
        T: TryInto<M2>,
    {
        SendError((self.0 .0.try_into().unwrap_silent(), self.0 .1))
    }

    pub(crate) fn map_cancel_first(self, output: T::Output) -> SendError<(T::Input, W)>
    where
        T: Message,
    {
        SendError((self.0 .0.cancel(output), self.0 .1))
    }
}

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

impl<T, W> TrySendError<(T, W)> {
    pub(crate) fn map_into_first(self) -> TrySendError<T> {
        match self {
            Self::Closed(t) => TrySendError::Closed(t.0),
            Self::Full(t) => TrySendError::Full(t.0),
        }
    }

    pub(crate) fn map_cancel_first(self, output: T::Output) -> TrySendError<(T::Input, W)>
    where
        T: Message,
    {
        match self {
            Self::Closed(t) => TrySendError::Closed((t.0.cancel(output), t.1)),
            Self::Full(t) => TrySendError::Full((t.0.cancel(output), t.1)),
        }
    }

    pub(crate) fn map_into_msg_first_unwrap<M>(self) -> TrySendError<(M, W)>
    where
        T: TryInto<M>,
    {
        match self {
            Self::Closed(t) => TrySendError::Closed((t.0.try_into().unwrap_silent(), t.1)),
            Self::Full(t) => TrySendError::Full((t.0.try_into().unwrap_silent(), t.1)),
        }
    }
}

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

    fn map<U>(self, f: impl FnOnce(T) -> U) -> DynSendError<U> {
        match self {
            Self::NotAccepted(t) => DynSendError::NotAccepted(f(t)),
            Self::Closed(t) => DynSendError::Closed(f(t)),
        }
    }
}

impl<T, W> DynSendError<(T, W)> {
    pub(crate) fn map_first(self) -> DynSendError<T> {
        match self {
            Self::NotAccepted(t) => DynSendError::NotAccepted(t.0),
            Self::Closed(t) => DynSendError::Closed(t.0),
        }
    }

    pub(crate) fn map_into_msg_unwrap<M2>(self) -> DynSendError<(M2, W)>
    where
        T: TryInto<M2>,
    {
        match self {
            Self::NotAccepted(t) => {
                DynSendError::NotAccepted((t.0.try_into().unwrap_silent(), t.1))
            }
            Self::Closed(t) => DynSendError::Closed((t.0.try_into().unwrap_silent(), t.1)),
        }
    }

    pub(crate) fn map_cancel_first(self, output: T::Output) -> DynSendError<(T::Input, W)>
    where
        T: Message,
    {
        match self {
            Self::NotAccepted(t) => DynSendError::NotAccepted((t.0.cancel(output), t.1)),
            Self::Closed(t) => DynSendError::Closed((t.0.cancel(output), t.1)),
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

    fn map<U>(self, f: impl FnOnce(T) -> U) -> DynTrySendError<U> {
        match self {
            Self::NotAccepted(t) => DynTrySendError::NotAccepted(f(t)),
            Self::Closed(t) => DynTrySendError::Closed(f(t)),
            Self::Full(t) => DynTrySendError::Full(f(t)),
        }
    }
}

impl<T, W> DynTrySendError<(T, W)> {
    pub(crate) fn map_first(self) -> DynTrySendError<T> {
        match self {
            Self::NotAccepted(t) => DynTrySendError::NotAccepted(t.0),
            Self::Closed(t) => DynTrySendError::Closed(t.0),
            Self::Full(t) => DynTrySendError::Full(t.0),
        }
    }

    pub(crate) fn map_cancel_first(self, output: T::Output) -> DynTrySendError<(T::Input, W)>
    where
        T: Message,
    {
        match self {
            Self::NotAccepted(t) => DynTrySendError::NotAccepted((t.0.cancel(output), t.1)),
            Self::Closed(t) => DynTrySendError::Closed((t.0.cancel(output), t.1)),
            Self::Full(t) => DynTrySendError::Full((t.0.cancel(output), t.1)),
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
