use crate::*;
use std::future::Future;

/// Any sender, whether dynamic or not, must implement this trait. It defines some basic common
/// methods for all senders, and defines the type [`IsSender::With`].
///
/// [`IsSender::With`] is the value that must be passed along when sending a message.
#[allow(clippy::len_without_is_empty)]
pub trait IsSender {
    /// The value that must be passed along when sending a message.
    type With;

    /// Returns `true` if the channel is closed.
    fn is_closed(&self) -> bool;

    /// Returns the capacity of the channel, if it is bounded.
    fn capacity(&self) -> Option<usize>;

    /// Returns the number of messages in the channel.
    fn len(&self) -> usize;

    /// Returns the number of receivers in the channel.
    fn receiver_count(&self) -> usize;

    /// Returns the number of senders in the channel.
    fn sender_count(&self) -> usize;
}

/// A supertrait of [`IsSender`], that additionally defines the protocol that can be sent to
/// this sender, and how to send it.
///
/// When this trait is implemented, [`Sends<M>`] is automatically implemented as well if
/// [`SendsProtocol::Protocol`] implements `From<M>` and `TryInto<M>`.
pub trait SendsProtocol: IsSender {
    /// The protocol that can be sent to this sender.
    type Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send;

    fn send_protocol_blocking_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> Result<(), SendError<(Self::Protocol, Self::With)>> {
        futures::executor::block_on(Self::send_protocol_with(this, protocol, with))
    }

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> Result<(), TrySendError<(Self::Protocol, Self::With)>>;
}

/// This trait defines when a message `M` can be sent to the sender.
///
/// Automatically implemented when [`SendsProtocol`] is implemented.
///
/// [`SendsExt`] is automatically implemented for all types that implement this trait, and contains
/// all the methods for sending messages.
pub trait Sends<M>: IsSender {
    fn send_msg_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> impl Future<Output = Result<(), SendError<(M, Self::With)>>> + Send;

    fn send_msg_blocking_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> Result<(), SendError<(M, Self::With)>> {
        futures::executor::block_on(Self::send_msg_with(this, msg, with))
    }

    fn try_send_msg_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> Result<(), TrySendError<(M, Self::With)>>;
}

impl<M, T> Sends<M> for T
where
    T: SendsProtocol,
    T::Protocol: From<M> + TryInto<M>,
{
    fn send_msg_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> impl Future<Output = Result<(), SendError<(M, Self::With)>>> + Send {
        let fut = Self::send_protocol_with(this, T::Protocol::from(msg), with);
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(t, w)| (t.try_into().unwrap_silent(), w))),
            }
        }
    }

    fn send_msg_blocking_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> Result<(), SendError<(M, Self::With)>> {
        T::send_protocol_blocking_with(this, T::Protocol::from(msg), with)
            .map_err(|e| e.map(|(t, w)| (t.try_into().unwrap_silent(), w)))
    }

    fn try_send_msg_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> Result<(), TrySendError<(M, Self::With)>> {
        T::try_send_protocol_with(this, T::Protocol::from(msg), with)
            .map_err(|e| e.map(|(t, w)| (t.try_into().unwrap_silent(), w)))
    }
}

/// Extension methods for [`IsSender`] / [`Sends<M>`].
pub trait SendsExt: IsSender {
    fn send_msg_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> impl Future<Output = Result<(), SendError<(M, Self::With)>>> + Send
    where
        Self: Sends<M>,
    {
        <Self as Sends<M>>::send_msg_with(self, msg, with)
    }
    fn send_msg_blocking_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> Result<(), SendError<(M, Self::With)>>
    where
        Self: Sends<M>,
    {
        <Self as Sends<M>>::send_msg_blocking_with(self, msg, with)
    }
    fn try_send_msg_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> Result<(), TrySendError<(M, Self::With)>>
    where
        Self: Sends<M>,
    {
        <Self as Sends<M>>::try_send_msg_with(self, msg, with)
    }
    fn send_msg<M: Message>(&self, msg: M) -> impl Future<Output = Result<(), SendError<M>>> + Send
    where
        Self: Sends<M>,
        Self::With: Default,
    {
        let fut = self.send_msg_with(msg, Default::default());
        async { fut.await.map_err(|e| e.map(|(t, _)| t)) }
    }
    fn send_msg_blocking<M: Message>(&self, msg: M) -> Result<(), SendError<M>>
    where
        Self: Sends<M>,
        Self::With: Default,
    {
        self.send_msg_blocking_with(msg, Default::default())
            .map_err(|e| e.map(|(t, _)| t))
    }
    fn try_send_msg<M: Message>(&self, msg: M) -> Result<(), TrySendError<M>>
    where
        Self: Sends<M>,
        Self::With: Default,
    {
        self.try_send_msg_with(msg, Default::default())
            .map_err(|e| e.map(|(t, _)| t))
    }

    fn send_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: Self::With,
    ) -> impl Future<Output = Result<M::Output, SendError<(M::Input, Self::With)>>> + Send
    where
        Self: Sends<M>,
        M::Output: Send,
    {
        let (msg, output) = M::create(msg.into());
        let fut = self.send_msg_with(msg, with);
        async {
            match fut.await {
                Ok(()) => Ok(output),
                Err(e) => Err(e.map(|(t, w)| (t.cancel(output), w))),
            }
        }
    }
    fn send_blocking_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: Self::With,
    ) -> Result<M::Output, SendError<(M::Input, Self::With)>>
    where
        Self: Sends<M>,
    {
        let (msg, output) = M::create(msg.into());
        match self.send_msg_blocking_with(msg, with) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.map(|(t, w)| (t.cancel(output), w))),
        }
    }
    fn try_send_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: Self::With,
    ) -> Result<M::Output, TrySendError<(M::Input, Self::With)>>
    where
        Self: Sends<M>,
    {
        let (msg, output) = M::create(msg.into());
        match self.try_send_msg_with(msg, with) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.map(|(t, w)| (t.cancel(output), w))),
        }
    }
    fn send<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> impl Future<Output = Result<M::Output, SendError<M::Input>>> + Send
    where
        Self: Sends<M>,
        Self::With: Default,
        M::Output: Send,
    {
        let fut = self.send_with(msg, Default::default());
        async { fut.await.map_err(|e| e.map(|(t, _)| t)) }
    }
    fn send_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, SendError<M::Input>>
    where
        Self: Sends<M>,
        Self::With: Default,
    {
        self.send_blocking_with(msg, Default::default())
            .map_err(|e| e.map(|(t, _)| t))
    }
    fn try_send<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, TrySendError<M::Input>>
    where
        Self: Sends<M>,
        Self::With: Default,
    {
        self.try_send_with(msg, Default::default())
            .map_err(|e| e.map(|(t, _)| t))
    }

    fn request_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: Self::With,
    ) -> impl std::future::Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<(M::Input, Self::With), <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: Sends<M>,
        M::Output: ResultFuture + Send,
    {
        let fut = self.send_with(msg, with);
        async {
            let rx = fut.await?;
            rx.await.map_err(RequestError::NoReply)
        }
    }
    fn request<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> impl std::future::Future<
        Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<M::Input, <M::Output as ResultFuture>::Error>,
        >,
    > + Send
    where
        Self: Sends<M>,
        Self::With: Default,
        M::Output: ResultFuture + Send,
    {
        let fut = self.request_with(msg, Default::default());
        async {
            fut.await.map_err(|e| match e {
                RequestError::Full(e) => RequestError::Full(e.0),
                RequestError::NoReply(e) => RequestError::NoReply(e),
            })
        }
    }
}
impl<T: ?Sized> SendsExt for T where T: IsSender {}

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
