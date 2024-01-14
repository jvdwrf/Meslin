use crate::*;
use futures::{future::BoxFuture, Future};
use std::{any::TypeId, marker::PhantomData};

// / DynSender![Ping, Pong with u32: !Clone]
pub struct DynSender<T, W = ()> {
    sender: Box<dyn DynSend<W>>,
    t: PhantomData<fn() -> T>,
}

pub trait DynSpecifier {
    type With;
}

pub trait DynSend<W = ()>: IsSender {
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<W>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<W>>>>;

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<W>,
    ) -> Result<(), DynSendError<BoxedMsg<W>>>;

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<W>,
    ) -> Result<(), DynTrySendError<BoxedMsg<W>>>;

    fn accepts_all(&self) -> &'static [TypeId];
}

impl<T, W> DynSend<W> for T
where
    T: SendProtocol<W> + Sync,
    T::Protocol: DynAccept<W>,
    W: 'static,
{
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<W>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<W>>>> {
        Box::pin(async move {
            let (protocol, with) =
                T::Protocol::try_from_boxed_msg(msg).map_err(DynSendError::NotAccepted)?;

            self.send_protocol_with(protocol, with)
                .await
                .map_err(|SendError((protocol, with))| {
                    DynSendError::Closed(protocol.into_boxed_msg(with))
                })
        })
    }

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<W>,
    ) -> Result<(), DynSendError<BoxedMsg<W>>> {
        let (protocol, with) =
            T::Protocol::try_from_boxed_msg(msg).map_err(DynSendError::NotAccepted)?;

        self.send_protocol_blocking_with(protocol, with)
            .map_err(|SendError((protocol, with))| {
                DynSendError::Closed(protocol.into_boxed_msg(with))
            })
    }

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<W>,
    ) -> Result<(), DynTrySendError<BoxedMsg<W>>> {
        let (protocol, with) =
            T::Protocol::try_from_boxed_msg(msg).map_err(DynTrySendError::NotAccepted)?;

        self.try_send_protocol_with(protocol, with)
            .map_err(|e| match e {
                TrySendError::Closed((protocol, with)) => {
                    DynTrySendError::Closed(protocol.into_boxed_msg(with))
                }
                TrySendError::Full((protocol, with)) => {
                    DynTrySendError::Full(protocol.into_boxed_msg(with))
                }
            })
    }

    fn accepts_all(&self) -> &'static [TypeId] {
        T::Protocol::accepts_all()
    }
}

pub trait DynSendExt<W = ()>: DynSend<W> {
    fn dyn_send_msg_with<M>(
        &self,
        msg: M,
        with: W,
    ) -> impl Future<Output = Result<(), DynSendError<(M, W)>>> + Send
    where
        M: Send + 'static,
        W: Send + 'static,
    {
        let fut = self.dyn_send_boxed_msg_with(BoxedMsg::new(msg, with));
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.downcast::<M>().unwrap_silent()),
            }
        }
    }

    fn dyn_send_msg_blocking_with<M>(&self, msg: M, with: W) -> Result<(), DynSendError<(M, W)>>
    where
        M: Send + 'static,
        W: Send + 'static,
    {
        match self.dyn_send_boxed_msg_blocking_with(BoxedMsg::new(msg, with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.downcast::<M>().unwrap_silent()),
        }
    }

    fn dyn_try_send_msg_with<M>(&self, msg: M, with: W) -> Result<(), DynTrySendError<(M, W)>>
    where
        M: Send + 'static,
        W: Send + 'static,
    {
        match self.dyn_try_send_boxed_msg_with(BoxedMsg::new(msg, with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.downcast::<M>().unwrap()),
        }
    }

    fn accepts<M: 'static>(&self) -> bool {
        self.accepts_all().contains(&TypeId::of::<M>())
    }
}
impl<T, W> DynSendExt<W> for T where T: DynSend<W> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynSendError<T> {
    NotAccepted(T),
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

impl<W: 'static> DynSendError<BoxedMsg<W>> {
    fn downcast<M: 'static>(self) -> Result<DynSendError<(M, W)>, Self> {
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

impl<T> From<NotAccepted<T>> for DynSendError<T> {
    fn from(NotAccepted(t): NotAccepted<T>) -> Self {
        Self::NotAccepted(t)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynTrySendError<T> {
    NotAccepted(T),
    Closed(T),
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

impl<W: 'static> DynTrySendError<BoxedMsg<W>> {
    fn downcast<M: 'static>(self) -> Result<DynTrySendError<(M, W)>, Self> {
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

impl<T> From<NotAccepted<T>> for DynTrySendError<T> {
    fn from(NotAccepted(t): NotAccepted<T>) -> Self {
        Self::NotAccepted(t)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAccepted<T>(pub T);
