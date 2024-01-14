use crate::{message::Message, protocol::ProtocolFor, ResultExt};
use std::future::Future;

// traits:
// - SendMessage<M>
// - SendMessageNow<M>
// - SendMessageWith<M>
// - SendMessageNowWith<M>

// - SendProtocol
// - SendProtocolNow
// - SendProtocolWith
// - SendProtocolNowWith

// - SendExt

//-------------------------------------
// SendsProtocol
//-------------------------------------

/// Send a message, and wait for space
pub trait SendProtocol<W = ()> {
    type Protocol;
    type Error;

    fn send_protocol_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> impl Future<Output = Result<(), Error<(Self::Protocol, W), Self::Error>>> + Send;

    fn send_protocol_blocking_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), Error<(Self::Protocol, W), Self::Error>> {
        futures::executor::block_on(Self::send_protocol_with(self, protocol, with))
    }
}

/// Send a message, and return an error if there is no space
pub trait SendProtocolNow<W = ()> {
    type Protocol;
    type Error;

    fn send_protocol_now_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), Error<(Self::Protocol, W), Self::Error>>;
}

//-------------------------------------
// SendsMessage
//-------------------------------------

/// This trait is implemented for all types that can send messages.
///
/// Usage is more convenient through [`Sends`] instead of using these methods directly.
pub trait SendMessage<M, W = ()>: Send + Sync {
    type Error;

    fn send_msg_with(
        &self,
        msg: M,
        with: W,
    ) -> impl Future<Output = Result<(), Error<(M, W), Self::Error>>> + Send;

    fn send_msg_blocking_with(
        &self,
        msg: M,
        with: W,
    ) -> Result<(), Error<(M, W), Self::Error>> {
        futures::executor::block_on(Self::send_msg_with(self, msg, with))
    }
}

impl<M, W, T> SendMessage<M, W> for T
where
    T: SendProtocol<W> + Send + Sync,
    T::Protocol: ProtocolFor<M>,
    M: Send,
    W: Send,
{
    type Error = <T as SendProtocol<W>>::Error;

    async fn send_msg_with(&self, msg: M, with: W) -> Result<(), Error<(M, W), Self::Error>> {
        self.send_protocol_with(T::Protocol::from_msg(msg), with)
            .await
            .map_err(|e| e.map_into_msg_unwrap())
    }

    fn send_msg_blocking_with(
        &self,
        msg: M,
        with: W,
    ) -> Result<(), Error<(M, W), Self::Error>> {
        self.send_protocol_blocking_with(T::Protocol::from_msg(msg), with)
            .map_err(|e| e.map_into_msg_unwrap())
    }
}

pub trait SendMessageNow<M, W = ()>: Send + Sync {
    type Error;

    fn send_msg_now_with(&self, msg: M, with: W) -> Result<(), Error<(M, W), Self::Error>>;
}

impl<M, W, T> SendMessageNow<M, W> for T
where
    T: SendProtocolNow<W> + Send + Sync,
    T::Protocol: ProtocolFor<M>,
    M: Send,
{
    type Error = <T as SendProtocolNow<W>>::Error;

    fn send_msg_now_with(&self, msg: M, with: W) -> Result<(), Error<(M, W), Self::Error>> {
        self.send_protocol_now_with(T::Protocol::from_msg(msg), with)
            .map_err(|e| e.map_into_msg_unwrap())
    }
}

//-------------------------------------
// SendExt
//-------------------------------------

/// Marker trait that can be implemented on any sender, to give it the [`SendExt`] methods.
pub trait SendExt {
    fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> impl Future<Output = Result<(), Error<Self::Protocol, Self::Error>>> + Send
    where
        Self: SendProtocol,
    {
        let fut = self.send_protocol_with(protocol, ());
        async { fut.await.map_err(|e| e.map_first()) }
    }

    fn send_protocol_blocking(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), Error<Self::Protocol, Self::Error>>
    where
        Self: SendProtocol,
    {
        self.send_protocol_blocking_with(protocol, ())
            .map_err(|e| e.map_first())
    }

    fn send_protocol_now(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), Error<Self::Protocol, Self::Error>>
    where
        Self: SendProtocolNow,
    {
        self.send_protocol_now_with(protocol, ())
            .map_err(|e| e.map_first())
    }

    fn send_msg<M: Message>(
        &self,
        msg: M,
    ) -> impl Future<Output = Result<(), Error<M, Self::Error>>> + Send
    where
        Self: SendMessage<M>,
    {
        let fut = self.send_msg_with(msg, ());
        async { fut.await.map_err(|e| e.map_first()) }
    }

    fn send_msg_blocking<M: Message>(&self, msg: M) -> Result<(), Error<M, Self::Error>>
    where
        Self: SendMessage<M>,
    {
        self.send_msg_blocking_with(msg, ())
            .map_err(|e| e.map_first())
    }

    fn send_msg_now<M: Message>(&self, msg: M) -> Result<(), Error<M, Self::Error>>
    where
        Self: SendMessageNow<M>,
    {
        self.send_msg_now_with(msg, ()).map_err(|e| e.map_first())
    }

    fn send_with<M: Message, W: Send>(
        &self,
        msg: impl Into<M::Input> + Send,
        with: W,
    ) -> impl Future<Output = Result<M::Output, Error<(M::Input, W), Self::Error>>> + Send
    where
        Self: SendMessage<M, W>,
    {
        async {
            let (msg, output) = M::create(msg.into());
            match self.send_msg_with(msg, with).await {
                Ok(()) => Ok(output),
                Err(e) => Err(e.map_cancel_first(output)),
            }
        }
    }

    fn send<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send,
    ) -> impl Future<Output = Result<M::Output, Error<M::Input, Self::Error>>> + Send
    where
        Self: SendMessage<M>,
    {
        async { self.send_with(msg, ()).await.map_err(|e| e.map_first()) }
    }

    fn send_blocking_with<M: Message, W: Send>(
        &self,
        msg: impl Into<M::Input>,
        with: W,
    ) -> Result<M::Output, Error<(M::Input, W), Self::Error>>
    where
        Self: SendMessage<M, W>,
    {
        let (msg, output) = M::create(msg.into());
        match self.send_msg_blocking_with(msg, with) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.map_cancel_first(output)),
        }
    }

    fn send_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, Error<M::Input, Self::Error>>
    where
        Self: SendMessage<M>,
    {
        self.send_blocking_with(msg, ()).map_err(|e| e.map_first())
    }

    fn send_now_with<M: Message, W: Send>(
        &self,
        msg: impl Into<M::Input>,
        with: W,
    ) -> Result<M::Output, Error<(M::Input, W), Self::Error>>
    where
        Self: SendMessageNow<M, W>,
    {
        let (msg, output) = M::create(msg.into());
        match self.send_msg_now_with(msg, with) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.map_cancel_first(output)),
        }
    }

    fn send_now<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, Error<M::Input, Self::Error>>
    where
        Self: SendMessageNow<M>,
    {
        self.send_now_with(msg, ()).map_err(|e| e.map_first())
    }

    fn request_with<M: Message, W: Send>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
        with: W,
    ) -> impl std::future::Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<Error<(M::Input, W), Self::Error>, <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: SendMessage<M, W>,
        M::Output: ResultFuture + Send + 'static,
    {
        async {
            let rx = self
                .send_with(msg, with)
                .await
                .map_err(RequestError::Send)?;
            rx.await.map_err(RequestError::Recv)
        }
    }

    fn request<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> impl std::future::Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<Error<M::Input, Self::Error>, <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: SendMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        async {
            self.request_with(msg, ()).await.map_err(|e| match e {
                RequestError::Send(e) => RequestError::Send(e.map_first()),
                RequestError::Recv(e) => RequestError::Recv(e),
            })
        }
    }

    fn request_blocking_with<M: Message, W: Send>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
        with: W,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestError<Error<(M::Input, W), Self::Error>, <M::Output as ResultFuture>::Error>,
    >
    where
        Self: SendMessage<M, W>,
        M::Output: ResultFuture + Send + 'static,
    {
        futures::executor::block_on(self.request_with(msg, with))
    }

    fn request_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestError<Error<M::Input, Self::Error>, <M::Output as ResultFuture>::Error>,
    >
    where
        Self: SendMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        futures::executor::block_on(self.request(msg))
    }
}

//-------------------------------------
// Errors
//-------------------------------------

#[derive(Debug)]
pub struct Error<T, E> {
    msg: T,
    reason: E,
}

impl<T, E> Error<T, E> {
    pub fn new(msg: T, reason: E) -> Self {
        Self { msg, reason }
    }

    pub fn reason(&self) -> &E {
        &self.reason
    }

    pub fn into_reason(self) -> E {
        self.reason
    }

    pub fn msg(&self) -> &T {
        &self.msg
    }

    pub fn into_msg(self) -> T {
        self.msg
    }

    pub fn into_inner(self) -> (T, E) {
        (self.msg, self.reason)
    }

    fn map_cancel(self, output: T::Output) -> Error<T::Input, E>
    where
        T: Message,
    {
        Error::new(T::cancel(self.msg, output), self.reason)
    }
}

impl<T, W, E> Error<(T, W), E> {
    fn map_into_msg_unwrap<M>(self) -> Error<(M, W), E>
    where
        T: ProtocolFor<M>,
    {
        Error::new(
            (self.msg.0.try_into_msg().unwrap_silent(), self.msg.1),
            self.reason,
        )
    }

    fn map_first(self) -> Error<T, E> {
        Error::new(self.msg.0, self.reason)
    }

    fn map_cancel_first(self, output: T::Output) -> Error<(T::Input, W), E>
    where
        T: Message,
    {
        Error::new((T::cancel(self.msg.0, output), self.msg.1), self.reason)
    }
}

#[derive(Debug)]
pub enum RequestError<E1, E2> {
    /// Error while sending the message
    Send(E1),
    /// Error while receiving the response
    Recv(E2),
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
