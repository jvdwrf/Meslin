use crate::*;
use std::future::Future;

//-------------------------------------
// SendsProtocol
//-------------------------------------

/// Send a message, and wait for space
pub trait SendProtocol<W = ()> {
    type Protocol;
    type SendError;
    type SendNowError;

    fn send_protocol_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> impl Future<Output = Result<(), Error<(Self::Protocol, W), Self::SendError>>> + Send;

    fn send_protocol_blocking_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), Error<(Self::Protocol, W), Self::SendError>> {
        futures::executor::block_on(Self::send_protocol_with(self, protocol, with))
    }

    fn send_protocol_now_with(
        &self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), Error<(Self::Protocol, W), Self::SendNowError>>;
}

//-------------------------------------
// SendsMessage
//-------------------------------------

/// This trait is implemented for all types that can send messages.
///
/// Usage is more convenient through [`Sends`] instead of using these methods directly.
pub trait SendMessage<M, W = ()>: Send + Sync {
    type SendError;
    type SendNowError;

    fn send_msg_with(
        &self,
        msg: M,
        with: W,
    ) -> impl Future<Output = Result<(), Error<(M, W), Self::SendError>>> + Send;

    fn send_msg_blocking_with(
        &self,
        msg: M,
        with: W,
    ) -> Result<(), Error<(M, W), Self::SendError>> {
        futures::executor::block_on(Self::send_msg_with(self, msg, with))
    }

    fn send_msg_now_with(&self, msg: M, with: W) -> Result<(), Error<(M, W), Self::SendNowError>>;
}

impl<M, W, T> SendMessage<M, W> for T
where
    T: SendProtocol<W> + Send + Sync,
    T::Protocol: Accept<M>,
    M: Send,
    W: Send,
{
    type SendError = <T as SendProtocol<W>>::SendError;
    type SendNowError = <T as SendProtocol<W>>::SendNowError;

    async fn send_msg_with(&self, msg: M, with: W) -> Result<(), Error<(M, W), Self::SendError>> {
        self.send_protocol_with(T::Protocol::from_msg(msg), with)
            .await
            .map_err(|e| e.map_into_msg_unwrap())
    }

    fn send_msg_blocking_with(
        &self,
        msg: M,
        with: W,
    ) -> Result<(), Error<(M, W), Self::SendError>> {
        self.send_protocol_blocking_with(T::Protocol::from_msg(msg), with)
            .map_err(|e| e.map_into_msg_unwrap())
    }

    fn send_msg_now_with(&self, msg: M, with: W) -> Result<(), Error<(M, W), Self::SendNowError>> {
        self.send_protocol_now_with(T::Protocol::from_msg(msg), with)
            .map_err(|e| e.map_into_msg_unwrap())
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
    ) -> impl Future<Output = Result<M::Output, Error<(M::Input, W), Self::SendError>>> + Send
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
    ) -> Result<M::Output, Error<(M::Input, W), Self::SendError>>
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
    ) -> Result<M::Output, Error<(M::Input, W), Self::SendNowError>>
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
            RequestError<Error<(M::Input, W), Self::SendError>, <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: SendMessage<M, W>,
        M::Output: ResultFuture + Send,
    {
        let fut = self.send_with(msg, with);
        async {
            let rx = fut.await.map_err(RequestError::Send)?;
            rx.await.map_err(RequestError::Recv)
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
    ) -> impl Future<Output = Result<(), Error<Self::Protocol, Self::SendError>>> + Send
    where
        Self: SendProtocol,
    {
        let fut = self.send_protocol_with(protocol, ());
        async { fut.await.map_err(|e| e.map_first()) }
    }

    fn send_protocol_blocking(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), Error<Self::Protocol, Self::SendError>>
    where
        Self: SendProtocol,
    {
        self.send_protocol_blocking_with(protocol, ())
            .map_err(|e| e.map_first())
    }

    fn send_protocol_now(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), Error<Self::Protocol, Self::SendNowError>>
    where
        Self: SendProtocol,
    {
        self.send_protocol_now_with(protocol, ())
            .map_err(|e| e.map_first())
    }

    fn send_msg<M: Message>(
        &self,
        msg: M,
    ) -> impl Future<Output = Result<(), Error<M, Self::SendError>>> + Send
    where
        Self: SendMessage<M>,
    {
        let fut = self.send_msg_with(msg, ());
        async { fut.await.map_err(|e| e.map_first()) }
    }

    fn send_msg_blocking<M: Message>(&self, msg: M) -> Result<(), Error<M, Self::SendError>>
    where
        Self: SendMessage<M>,
    {
        self.send_msg_blocking_with(msg, ())
            .map_err(|e| e.map_first())
    }

    fn send_msg_now<M: Message>(&self, msg: M) -> Result<(), Error<M, Self::SendNowError>>
    where
        Self: SendMessage<M>,
    {
        self.send_msg_now_with(msg, ()).map_err(|e| e.map_first())
    }

    fn send<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send,
    ) -> impl Future<Output = Result<M::Output, Error<M::Input, Self::SendError>>> + Send
    where
        Self: SendMessage<M>,
    {
        async { self.send_with(msg, ()).await.map_err(|e| e.map_first()) }
    }

    fn send_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, Error<M::Input, Self::SendError>>
    where
        Self: SendMessage<M>,
    {
        self.send_blocking_with(msg, ()).map_err(|e| e.map_first())
    }

    fn send_now<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, Error<M::Input, Self::SendNowError>>
    where
        Self: SendMessage<M>,
    {
        self.send_now_with(msg, ()).map_err(|e| e.map_first())
    }

    fn request<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> impl std::future::Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<Error<M::Input, Self::SendError>, <M::Output as ResultFuture>::Error>,
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

    fn request_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestError<Error<M::Input, Self::SendError>, <M::Output as ResultFuture>::Error>,
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
}

impl<T, W, E> Error<(T, W), E> {
    fn map_into_msg_unwrap<M>(self) -> Error<(M, W), E>
    where
        T: Accept<M>,
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
