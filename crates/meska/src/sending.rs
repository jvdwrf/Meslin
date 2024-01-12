use crate::{message::Message, protocol::ProtocolFor, ResultExt};
use std::future::Future;

//-------------------------------------
// SendsMessage
//-------------------------------------

/// This trait is implemented for all types that can send messages.
///
/// Usage is more convenient through [`Sends`] instead of using these methods directly.
pub trait SendsMessage<M>: Send + Sync {
    fn send_msg(&self, msg: M) -> impl Future<Output = Result<(), SendError<M>>> + Send;

    fn try_send_msg(&self, msg: M) -> Result<(), TrySendError<M>>;

    fn send_msg_blocking(&self, msg: M) -> Result<(), SendError<M>> {
        futures::executor::block_on(Self::send_msg(self, msg))
    }
}

//-------------------------------------
// SendsProtocol
//-------------------------------------

pub trait SendsProtocol: Send + Sync {
    type Protocol;
    fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> impl Future<Output = Result<(), SendError<Self::Protocol>>> + Send;

    fn try_send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), crate::sending::TrySendError<Self::Protocol>>;

    fn send_protocol_blocking(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol>> {
        futures::executor::block_on(Self::send_protocol(self, protocol))
    }
}

impl<M: Send, T> SendsMessage<M> for T
where
    T: SendsProtocol,
    T::Protocol: ProtocolFor<M>,
{
    async fn send_msg(&self, msg: M) -> Result<(), SendError<M>> {
        self.send_protocol(T::Protocol::from_msg(msg))
            .await
            .map_err(|SendError(protocol)| SendError(protocol.try_into_msg().unwrap_silent()))
    }

    fn try_send_msg(&self, msg: M) -> Result<(), crate::sending::TrySendError<M>> {
        self.try_send_protocol(T::Protocol::from_msg(msg))
            .map_err(|e| match e {
                TrySendError::Closed(protocol) => {
                    TrySendError::Closed(protocol.try_into_msg().unwrap_silent())
                }
                TrySendError::Full(protocol) => {
                    TrySendError::Full(protocol.try_into_msg().unwrap_silent())
                }
            })
    }

    fn send_msg_blocking(&self, msg: M) -> Result<(), SendError<M>> {
        self.send_protocol_blocking(T::Protocol::from_msg(msg))
            .map_err(|SendError(protocol)| SendError(protocol.try_into_msg().unwrap_silent()))
    }
}

//-------------------------------------
// SendExt
//-------------------------------------

/// Automatically implemented for all types, providing convenience functions
/// for the trait [`SendsMessage<M>`].
pub trait SendsExt {
    fn send<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send,
    ) -> impl Future<Output = Result<M::Output, SendError<M::Input>>> + Send
    where
        Self: SendsMessage<M>,
    {
        async {
            let (msg, output) = M::create(msg.into());
            match self.send_msg(msg).await {
                Ok(()) => Ok(output),
                Err(e) => Err(e.cancel(output)),
            }
        }
    }

    fn try_send<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, TrySendError<M::Input>>
    where
        Self: SendsMessage<M>,
    {
        let (msg, output) = M::create(msg.into());
        match self.try_send_msg(msg) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.cancel(output)),
        }
    }

    fn send_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, SendError<M::Input>>
    where
        Self: SendsMessage<M>,
    {
        let (msg, output) = M::create(msg.into());
        match self.send_msg_blocking(msg) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.cancel(output)),
        }
    }

    fn request<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> impl std::future::Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<SendError<M::Input>, <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: SendsMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        async {
            let rx = self.send(msg).await.map_err(RequestError::Send)?;
            rx.await.map_err(RequestError::Recv)
        }
    }

    fn try_request<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> impl Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<TrySendError<M::Input>, <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: SendsMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        async {
            let rx = self.try_send(msg).map_err(RequestError::Send)?;
            rx.await.map_err(RequestError::Recv)
        }
    }

    fn request_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestError<SendError<M::Input>, <M::Output as ResultFuture>::Error>,
    >
    where
        Self: SendsMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        futures::executor::block_on(self.request(msg))
    }

    fn try_request_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestError<TrySendError<M::Input>, <M::Output as ResultFuture>::Error>,
    >
    where
        Self: SendsMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        futures::executor::block_on(self.try_request(msg))
    }
}
impl<T: ?Sized> SendsExt for T {}

//-------------------------------------
// Errors
//-------------------------------------

#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> SendError<T> {
    fn cancel(self, output: T::Output) -> SendError<T::Input>
    where
        T: Message,
    {
        SendError(T::cancel(self.0, output))
    }

    pub fn into_msg(self) -> T {
        self.0
    }

    pub fn msg(&self) -> &T {
        &self.0
    }
}

#[derive(Debug)]
pub enum TrySendError<T> {
    Closed(T),
    Full(T),
}

impl<T> TrySendError<T> {
    fn cancel(self, output: T::Output) -> TrySendError<T::Input>
    where
        T: Message,
    {
        match self {
            TrySendError::Closed(e) => TrySendError::Closed(T::cancel(e, output)),
            TrySendError::Full(e) => TrySendError::Full(T::cancel(e, output)),
        }
    }

    pub fn into_msg(self) -> T {
        match self {
            TrySendError::Closed(e) => e,
            TrySendError::Full(e) => e,
        }
    }

    pub fn msg(&self) -> &T {
        match self {
            TrySendError::Closed(e) => e,
            TrySendError::Full(e) => e,
        }
    }
}

#[derive(Debug)]
pub enum RequestError<S, R> {
    Send(S),
    Recv(R),
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
