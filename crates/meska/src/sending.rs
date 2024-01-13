use crate::{message::Message, protocol::ProtocolFor, ResultExt};
use std::future::Future;

//-------------------------------------
// SendsMessage
//-------------------------------------

/// This trait is implemented for all types that can send messages.
///
/// Usage is more convenient through [`Sends`] instead of using these methods directly.
pub trait SendMessage<M>: Send + Sync {
    type Error;

    fn send_msg(
        &self,
        msg: M,
    ) -> impl Future<Output = Result<(), SendError<M, Self::Error>>> + Send;

    fn send_msg_blocking(&self, msg: M) -> Result<(), SendError<M, Self::Error>> {
        futures::executor::block_on(Self::send_msg(self, msg))
    }
}

pub trait SendMessageNow<M>: Send + Sync {
    type Error;

    fn send_msg_now(&self, msg: M) -> Result<(), SendError<M, Self::Error>>;
}

//-------------------------------------
// SendsProtocol
//-------------------------------------

/// Send a message, and wait for space
pub trait SendProtocol {
    type Protocol;
    type Error;

    fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> impl Future<Output = Result<(), SendError<Self::Protocol, Self::Error>>> + Send;

    fn send_protocol_blocking(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol, Self::Error>> {
        futures::executor::block_on(Self::send_protocol(self, protocol))
    }
}

/// Send a message, and return an error if there is no space
pub trait SendProtocolNow {
    type Protocol;
    type Error;

    fn send_protocol_now(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol, Self::Error>>;
}

impl<M: Send, T> SendMessage<M> for T
where
    T: SendProtocol + Send + Sync,
    T::Protocol: ProtocolFor<M>,
{
    type Error = <T as SendProtocol>::Error;

    async fn send_msg(&self, msg: M) -> Result<(), SendError<M, Self::Error>> {
        self.send_protocol(T::Protocol::from_msg(msg))
            .await
            .map_err(|e| {
                let (m, e) = e.into_inner();
                SendError::new(m.try_into_msg().unwrap_silent(), e)
            })
    }

    fn send_msg_blocking(&self, msg: M) -> Result<(), SendError<M, Self::Error>> {
        self.send_protocol_blocking(T::Protocol::from_msg(msg))
            .map_err(|e| {
                let (m, e) = e.into_inner();
                SendError::new(m.try_into_msg().unwrap_silent(), e)
            })
    }
}

impl<M: Send, T> SendMessageNow<M> for T
where
    T: SendProtocolNow + Send + Sync,
    T::Protocol: ProtocolFor<M>,
{
    type Error = <T as SendProtocolNow>::Error;

    fn send_msg_now(&self, msg: M) -> Result<(), SendError<M, Self::Error>> {
        self.send_protocol_now(T::Protocol::from_msg(msg))
            .map_err(|e| {
                let (m, e) = e.into_inner();
                SendError::new(m.try_into_msg().unwrap_silent(), e)
            })
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
    ) -> impl Future<Output = Result<M::Output, SendError<M::Input, Self::Error>>> + Send
    where
        Self: SendMessage<M>,
    {
        async {
            let (msg, output) = M::create(msg.into());
            match self.send_msg(msg).await {
                Ok(()) => Ok(output),
                Err(e) => Err(e.cancel(output)),
            }
        }
    }

    fn send_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, SendError<M::Input, Self::Error>>
    where
        Self: SendMessage<M>,
    {
        let (msg, output) = M::create(msg.into());
        match self.send_msg_blocking(msg) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.cancel(output)),
        }
    }

    fn send_now<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, SendError<M::Input, Self::Error>>
    where
        Self: SendMessageNow<M>,
    {
        let (msg, output) = M::create(msg.into());
        match self.send_msg_now(msg) {
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
            RequestError<SendError<M::Input, Self::Error>, <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: SendMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        async {
            let rx = self.send(msg).await.map_err(RequestError::Send)?;
            rx.await.map_err(RequestError::Recv)
        }
    }

    fn request_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestError<SendError<M::Input, Self::Error>, <M::Output as ResultFuture>::Error>,
    >
    where
        Self: SendMessage<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        futures::executor::block_on(self.request(msg))
    }
}
impl<T: ?Sized> SendsExt for T {}

//-------------------------------------
// Errors
//-------------------------------------

#[derive(Debug)]
pub struct SendError<M, E> {
    msg: M,
    reason: E,
}

impl<M, E> SendError<M, E> {
    pub fn new(msg: M, reason: E) -> Self {
        Self { msg, reason }
    }

    pub fn reason(&self) -> &E {
        &self.reason
    }

    pub fn into_reason(self) -> E {
        self.reason
    }

    pub fn msg(&self) -> &M {
        &self.msg
    }

    pub fn into_msg(self) -> M {
        self.msg
    }

    pub fn into_inner(self) -> (M, E) {
        (self.msg, self.reason)
    }

    pub fn cancel(self, output: M::Output) -> SendError<M::Input, E>
    where
        M: Message,
    {
        SendError::new(M::cancel(self.msg, output), self.reason)
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
