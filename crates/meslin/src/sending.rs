use crate::*;
use std::future::Future;

//-------------------------------------
// SendsProtocol
//-------------------------------------

/// Send a message, and wait for space
pub trait SendProtocol<W = ()> {
    type Protocol;

    fn send_protocol_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, W)>>> + Send;

    fn send_protocol_blocking_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), SendError<(Self::Protocol, W)>> {
        futures::executor::block_on(self.send_protocol_with(protocol, with))
    }

    fn send_protocol_now_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), SendNowError<(Self::Protocol, W)>>;
}

//-------------------------------------
// SendsMessage
//-------------------------------------

/// This trait is implemented for all types that can send messages.
///
/// Usage is more convenient through [`Sends`] instead of using these methods directly.
pub trait SendMessage<M, W = ()>: Send + Sync {
    fn send_msg_with(
        &self,
        msg: M,
        with: W,
    ) -> impl Future<Output = Result<(), SendError<(M, W)>>> + Send;

    fn send_msg_blocking_with(&self, msg: M, with: W) -> Result<(), SendError<(M, W)>> {
        futures::executor::block_on(Self::send_msg_with(self, msg, with))
    }

    fn send_msg_now_with(&self, msg: M, with: W) -> Result<(), SendNowError<(M, W)>>;
}

impl<M, W, T> SendMessage<M, W> for T
where
    T: SendProtocol<W> + Send + Sync,
    T::Protocol: Accept<M>,
    M: Send,
    W: Send,
{
    async fn send_msg_with(&self, msg: M, with: W) -> Result<(), SendError<(M, W)>> {
        self.send_protocol_with(T::Protocol::from_msg(msg), with)
            .await
            .map_err(|e| e.map_into_msg_unwrap())
    }

    fn send_msg_blocking_with(&self, msg: M, with: W) -> Result<(), SendError<(M, W)>> {
        self.send_protocol_blocking_with(T::Protocol::from_msg(msg), with)
            .map_err(|e| e.map_into_msg_unwrap())
    }

    fn send_msg_now_with(&self, msg: M, with: W) -> Result<(), SendNowError<(M, W)>> {
        self.send_protocol_now_with(T::Protocol::from_msg(msg), with)
            .map_err(|e| e.map_into_msg_first_unwrap())
    }
}

//-------------------------------------
// IsSender
//-------------------------------------

#[allow(clippy::len_without_is_empty)]
pub trait IsSender<W = ()> {
    fn is_closed(&self) -> bool;
    fn capacity(&self) -> Option<usize>;
    fn len(&self) -> usize;
    fn receiver_count(&self) -> usize;
    fn sender_count(&self) -> usize;
}

//-------------------------------------
// SendExt
//-------------------------------------

/// A marker trait that should be implemented on any sender, to give it the [`SendWith`] and
/// [`SendExt`] methods.
pub trait SendWith<W = ()> {
    fn send_with<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send,
        with: W,
    ) -> impl Future<Output = Result<M::Output, SendError<(M::Input, W)>>> + Send
    where
        Self: SendMessage<M, W>,
    {
        let (msg, output) = M::create(msg.into());
        let fut = self.send_msg_with(msg, with);
        async {
            match fut.await {
                Ok(()) => Ok(output),
                Err(e) => Err(e.map_cancel_first(output)),
            }
        }
    }

    fn send_blocking_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: W,
    ) -> Result<M::Output, SendError<(M::Input, W)>>
    where
        Self: SendMessage<M, W>,
    {
        let (msg, output) = M::create(msg.into());
        match self.send_msg_blocking_with(msg, with) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.map_cancel_first(output)),
        }
    }

    fn send_now_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: W,
    ) -> Result<M::Output, SendNowError<(M::Input, W)>>
    where
        Self: SendMessage<M, W>,
    {
        let (msg, output) = M::create(msg.into());
        match self.send_msg_now_with(msg, with) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.map_cancel_first(output)),
        }
    }

    fn request_with<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send,
        with: W,
    ) -> impl std::future::Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<(M::Input, W), <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: SendMessage<M, W>,
        M::Output: ResultFuture + Send,
    {
        let fut = self.send_with(msg, with);
        async {
            let rx = fut.await?;
            rx.await.map_err(RequestError::NoReply)
        }
    }
}
impl<T, W> SendWith<W> for T where T: IsSender<W> {}

/// An extension to [`SendWith`] that provides more convenient methods.
///
/// For implementation, use [`SendWith`] instead. It automatically implements this trait.
pub trait SendExt: SendWith {
    fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> impl Future<Output = Result<(), SendError<Self::Protocol>>> + Send
    where
        Self: SendProtocol,
    {
        let fut = self.send_protocol_with(protocol, ());
        async { fut.await.map_err(|e| e.map_first()) }
    }

    fn send_protocol_blocking(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol>>
    where
        Self: SendProtocol,
    {
        self.send_protocol_blocking_with(protocol, ())
            .map_err(|e| e.map_first())
    }

    fn send_protocol_now(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendNowError<Self::Protocol>>
    where
        Self: SendProtocol,
    {
        self.send_protocol_now_with(protocol, ())
            .map_err(|e| e.map_into_first())
    }

    fn send_msg<M: Message>(&self, msg: M) -> impl Future<Output = Result<(), SendError<M>>> + Send
    where
        Self: SendMessage<M>,
    {
        let fut = self.send_msg_with(msg, ());
        async { fut.await.map_err(|e| e.map_first()) }
    }

    fn send_msg_blocking<M: Message>(&self, msg: M) -> Result<(), SendError<M>>
    where
        Self: SendMessage<M>,
    {
        self.send_msg_blocking_with(msg, ())
            .map_err(|e| e.map_first())
    }

    fn send_msg_now<M: Message>(&self, msg: M) -> Result<(), SendNowError<M>>
    where
        Self: SendMessage<M>,
    {
        self.send_msg_now_with(msg, ())
            .map_err(|e| e.map_into_first())
    }

    fn send<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send,
    ) -> impl Future<Output = Result<M::Output, SendError<M::Input>>> + Send
    where
        Self: SendMessage<M>,
    {
        async { self.send_with(msg, ()).await.map_err(|e| e.map_first()) }
    }

    fn send_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, SendError<M::Input>>
    where
        Self: SendMessage<M>,
    {
        self.send_blocking_with(msg, ()).map_err(|e| e.map_first())
    }

    fn send_now<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, SendNowError<M::Input>>
    where
        Self: SendMessage<M>,
    {
        self.send_now_with(msg, ()).map_err(|e| e.map_into_first())
    }

    fn request<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> impl std::future::Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<M::Input, <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: SendMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        async {
            self.request_with(msg, ()).await.map_err(|e| match e {
                RequestError::Full(e) => RequestError::Full(e.0),
                RequestError::NoReply(e) => RequestError::NoReply(e),
            })
        }
    }

    fn request_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestError<M::Input, <M::Output as ResultFuture>::Error>,
    >
    where
        Self: SendMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        futures::executor::block_on(self.request(msg))
    }
}
impl<T> SendExt for T where T: SendWith {}

//-------------------------------------
// Errors
//-------------------------------------

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SendError<T>(pub T);

impl<T> SendError<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, W> SendError<(T, W)> {
    fn map_first(self) -> SendError<T> {
        SendError(self.0 .0)
    }

    fn map_into_msg_unwrap<M2>(self) -> SendError<(M2, W)>
    where
        T: Accept<M2>,
    {
        SendError((self.0 .0.try_into_msg().unwrap_silent(), self.0 .1))
    }

    fn map_cancel_first(self, output: T::Output) -> SendError<(T::Input, W)>
    where
        T: Message,
    {
        SendError((self.0 .0.cancel(output), self.0 .1))
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SendNowError<T> {
    Closed(T),
    Full(T),
}

impl<T> SendNowError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::Closed(t) => t,
            Self::Full(t) => t,
        }
    }
}

impl<T, W> SendNowError<(T, W)> {
    fn map_into_first(self) -> SendNowError<T> {
        match self {
            Self::Closed(t) => SendNowError::Closed(t.0),
            Self::Full(t) => SendNowError::Full(t.0),
        }
    }

    fn map_cancel_first(self, output: T::Output) -> SendNowError<(T::Input, W)>
    where
        T: Message,
    {
        match self {
            Self::Closed(t) => SendNowError::Closed((t.0.cancel(output), t.1)),
            Self::Full(t) => SendNowError::Full((t.0.cancel(output), t.1)),
        }
    }

    fn map_into_msg_first_unwrap<M>(self) -> SendNowError<(M, W)>
    where
        T: Accept<M>,
    {
        match self {
            Self::Closed(t) => SendNowError::Closed((t.0.try_into_msg().unwrap_silent(), t.1)),
            Self::Full(t) => SendNowError::Full((t.0.try_into_msg().unwrap_silent(), t.1)),
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

//-------------------------------------
// ResultFuture
//-------------------------------------

pub trait ResultFuture: Future<Output = Result<Self::Ok, Self::Error>> {
    type Error;
    type Ok;
}

impl<T, O, E> ResultFuture for T
where
    T: Future<Output = Result<O, E>>,
{
    type Error = E;
    type Ok = O;
}
