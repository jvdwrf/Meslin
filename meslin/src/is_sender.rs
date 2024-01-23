use futures::future::BoxFuture;

use crate::*;
use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

/// Trait that must be implemented by all senders.
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

/// A supertrait of [`IsSender`], that defines how a protocol can be sent to the sender.
///
/// When this trait is implemented, [`Sends<M>`] is automatically implemented as well if
/// [`SendsProtocol::Protocol`] implements `From<M>` and `TryInto<M>`.
pub trait IsStaticSender: IsSender {
    /// The protocol that can be sent to this sender.
    type Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send;

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> Result<(), SendNowError<(Self::Protocol, Self::With)>>;

    fn send_protocol_blocking_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> Result<(), SendError<(Self::Protocol, Self::With)>> {
        futures::executor::block_on(Self::send_protocol_with(this, protocol, with))
    }
}

/// Defines when a message `M` can be sent to the sender.
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
    ) -> Result<(), SendNowError<(M, Self::With)>>;
}

impl<M, T> Sends<M> for T
where
    T: IsStaticSender,
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
    ) -> Result<(), SendNowError<(M, Self::With)>> {
        T::try_send_protocol_with(this, T::Protocol::from(msg), with)
            .map_err(|e| e.map(|(t, w)| (t.try_into().unwrap_silent(), w)))
    }
}

/// Extension methods for [`IsSender`].
pub trait IsSenderExt: IsSender + Sized {
    /// Map the `with` value of the sender to `()`, by providing the default `with` to use.
    fn with(self, with: Self::With) -> WithValueSender<Self> {
        WithValueSender::new(self, with)
    }

    /// Map the `with` value of the sender to `W`, by providing conversion functions.
    fn map_with<W, F1, F2>(self, f1: F1, f2: F2) -> MappedWithSender<Self, W, F1, F2>
    where
        F1: Fn(W) -> Self::With,
        F2: Fn(Self::With) -> W,
    {
        MappedWithSender::new(self, f1, f2)
    }

    fn send<M: Message>(&self, msg: impl Into<M::Input>) -> SendFut<'_, Self, M> {
        SendFut::new(self, msg.into())
    }

    fn send_msg<M>(&self, msg: M) -> SendMsgFut<'_, Self, M> {
        SendMsgFut::new(self, msg)
    }
}
impl<T: ?Sized> IsSenderExt for T where T: IsSender + Sized {}

//-------------------------------------
// ResultFuture
//-------------------------------------

/// A future that resolves to a [`Result`].
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
