use crate::*;
use futures::{future::BoxFuture, Future};
use std::{any::TypeId, marker::PhantomData};

/// DynSender<Accepts![Ping, Pong], u32>
/// DynSender<NoClone<AcceptTwo<Ping, Pong>>, u32>
pub struct DynSender<T, W = ()> {
    sender: Box<dyn IsDynSender<With = W>>,
    t: PhantomData<fn() -> T>,
}

impl<T, W> Clone for DynSender<T, W> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            t: PhantomData,
        }
    }
}

pub trait IsDynSender: IsSender {
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>>;

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>>;

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynTrySendError<BoxedMsg<Self::With>>>;

    fn accepts_all(&self) -> &'static [TypeId];

    fn clone_boxed(&self) -> Box<dyn IsDynSender<With = Self::With>>;
}

impl<T> IsDynSender for T
where
    T: SendsProtocol + Clone + Sync + 'static,
    T::Protocol: BoxedFromInto,
    T::With: Send
{
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>> {
        Box::pin(async move {
            let (protocol, with) =
                T::Protocol::try_from_boxed_msg(msg).map_err(DynSendError::NotAccepted)?;

            T::send_protocol_with(self, protocol, with)
                .await
                .map_err(|SendError((protocol, with))| {
                    DynSendError::Closed(protocol.into_boxed_msg(with))
                })
        })
    }

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>> {
        let (protocol, with) =
            T::Protocol::try_from_boxed_msg(msg).map_err(DynSendError::NotAccepted)?;

        T::send_protocol_blocking_with(self, protocol, with)
            .map_err(|SendError((protocol, with))| {
                DynSendError::Closed(protocol.into_boxed_msg(with))
            })
    }

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynTrySendError<BoxedMsg<Self::With>>> {
        let (protocol, with) =
            T::Protocol::try_from_boxed_msg(msg).map_err(DynTrySendError::NotAccepted)?;

        T::try_send_protocol_with(self, protocol, with)
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

    fn clone_boxed(&self) -> Box<dyn IsDynSender<With = Self::With>> {
        Box::new(self.clone())
    }
}

impl<T> Clone for Box<dyn IsDynSender<With = T>> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

pub trait DynSendExt: IsDynSender {
    fn dyn_send_msg_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> impl Future<Output = Result<(), DynSendError<(M, Self::With)>>> + Send
    where
        M: Send + 'static,
        Self::With: Send + 'static,
    {
        let fut = self.dyn_send_boxed_msg_with(BoxedMsg::new(msg, with));
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.downcast::<M>().unwrap_silent()),
            }
        }
    }

    fn dyn_send_msg_blocking_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> Result<(), DynSendError<(M, Self::With)>>
    where
        M: Send + 'static,
        Self::With: Send + 'static,
    {
        match self.dyn_send_boxed_msg_blocking_with(BoxedMsg::new(msg, with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.downcast::<M>().unwrap_silent()),
        }
    }

    fn dyn_try_send_msg_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> Result<(), DynTrySendError<(M, Self::With)>>
    where
        M: Send + 'static,
        Self::With: Send + 'static,
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
impl<T> DynSendExt for T where T: IsDynSender {}

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
