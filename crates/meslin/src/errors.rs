use crate::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SendError<T>(pub T);

impl<T> SendError<T> {
    pub fn into_inner(self) -> T {
        self.0
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TrySendError<T> {
    Closed(T),
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

#[derive(Debug)]
pub enum RequestError<M, E> {
    Full(M),
    NoReply(E),
}

impl<T, E> From<SendError<T>> for RequestError<T, E> {
    fn from(e: SendError<T>) -> Self {
        Self::Full(e.0)
    }
}
