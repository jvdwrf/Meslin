use crate::{
    message::{Protocol, Message},
    AnyBox,
};
use futures::{executor::block_on, Future, Stream, StreamExt};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};

/// A child specification describes how a child is spawned, supervised
/// and exits.
pub trait ChildSpec: Unpin {
    /// The optional config this child is spawned with.
    type Config;

    /// The output of this child when it exits.
    type Output;

    /// The error that can occur when polling this child.
    type OutputError;

    /// Spawn a future with the given config, returning the child.
    fn spawn_future<F>(cfg: Option<Self::Config>, f: F) -> Self
    where
        F: Future<Output = Self::Output> + Send + 'static,
        F::Output: Send + 'static;

    /// Poll the child for it's exit.
    fn poll_child_exit(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Output, Self::OutputError>>;
}

pub trait AddressSpec {
    type Protocol;
    type Output;

    fn is_alive(&self) -> bool;

    fn poll_address(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;

    fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> impl Future<Output = Result<(), SendError<Self::Protocol>>> + Send + '_;

    fn try_send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), TrySendError<Self::Protocol>>;

    fn send_protocol_blocking(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol>> {
        block_on(Self::send_protocol(self, protocol))
    }
}

pub trait DynAddressSpec: AddressSpec {
    fn accepts_msg(&self, msg_type_id: TypeId) -> bool;
    fn send_msg_dyn<M: Message + Send + 'static>(
        &self,
        payload: M,
    ) -> impl Future<Output = Result<(), SendDynError<M>>> + Send + '_;
    fn send_msg_dyn_blocking<M: Message + Send + 'static>(
        &self,
        payload: M,
    ) -> Result<(), SendDynError<M>>;
    fn try_send_msg_dyn<M: Message + Send + 'static>(
        &self,
        payload: M,
    ) -> Result<(), TrySendDynError<M>>;
}

pub trait StateSpec {
    type State;
    fn state(&self) -> &Self::State;
}

pub trait InboxSpec: Stream<Item = Self::Receives> + Send + Unpin + Sized {
    type Receives;

    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<Self::Receives, ReceiveError>> + Send + '_ {
        let next = self.next();
        async move { next.await.ok_or(ReceiveError::Closed) }
    }
}

pub trait ChannelSpec {
    type Config;
    type InboxSpec: InboxSpec;
    type AddressSpec: AddressSpec;
    fn create(with: Option<Self::Config>) -> (Self::InboxSpec, Self::AddressSpec);
}

pub trait FromSpec<T> {
    fn from_spec(spec: T) -> Self;
}

pub trait IntoSpec<T> {
    fn into_spec(self) -> T;
}

impl<T, R> IntoSpec<R> for T
where
    R: FromSpec<T>,
{
    fn into_spec(self) -> R {
        R::from_spec(self)
    }
}

pub enum ReceiveError {
    Closed,
}

#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> SendError<T> {
    pub fn cancel(self, returned: T::Output) -> SendError<T::Input>
    where
        T: Message,
    {
        SendError(T::cancel(self.0, returned))
    }

    pub fn cancel_protocol<M: Message>(self, output: M::Output) -> SendError<M::Input>
    where
        T: Protocol<M>,
    {
        let Ok(msg) = self.0.try_into_msg() else {
            panic!("Cannot cancel protocol with incompatible message type.")
        };
        SendError(M::cancel(msg, output))
    }

    pub fn into_msg(self) -> T {
        self.0
    }

    pub fn msg(&self) -> &T {
        &self.0
    }
}

pub enum TrySendError<T> {
    Closed(T),
    Full(T),
}

impl<T> TrySendError<T> {
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
pub enum SendDynError<T> {
    Closed(T),
    NotAccepted(T),
}

#[derive(Debug)]
pub enum TrySendDynError<T> {
    Closed(T),
    Full(T),
    NotAccepted(T),
}

impl<T> SendDynError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::Closed(t) => t,
            Self::NotAccepted(t) => t,
        }
    }

    pub fn cancel(self, output: T::Output) -> SendDynError<T::Input>
    where
        T: Message,
    {
        match self {
            Self::Closed(t) => SendDynError::Closed(T::cancel(t, output)),
            Self::NotAccepted(t) => SendDynError::NotAccepted(T::cancel(t, output)),
        }
    }

    pub fn boxed(self) -> SendDynError<AnyBox>
    where
        T: Any + Send + 'static,
    {
        match self {
            Self::Closed(t) => SendDynError::Closed(Box::new(t)),
            Self::NotAccepted(t) => SendDynError::NotAccepted(Box::new(t)),
        }
    }
}

impl<T> TrySendDynError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::Closed(t) => t,
            Self::Full(t) => t,
            Self::NotAccepted(t) => t,
        }
    }

    pub fn cancel(self, output: T::Output) -> TrySendDynError<T::Input>
    where
        T: Message,
    {
        match self {
            Self::Closed(t) => TrySendDynError::Closed(T::cancel(t, output)),
            Self::Full(t) => TrySendDynError::Full(T::cancel(t, output)),
            Self::NotAccepted(t) => TrySendDynError::NotAccepted(T::cancel(t, output)),
        }
    }

    pub fn boxed(self) -> TrySendDynError<AnyBox>
    where
        T: Any + Send + 'static,
    {
        match self {
            Self::Closed(t) => TrySendDynError::Closed(Box::new(t)),
            Self::Full(t) => TrySendDynError::Full(Box::new(t)),
            Self::NotAccepted(t) => TrySendDynError::NotAccepted(Box::new(t)),
        }
    }
}

impl SendDynError<AnyBox> {
    pub fn downcast<M: Message + 'static>(self) -> Result<SendDynError<M>, SendDynError<AnyBox>> {
        match self {
            Self::Closed(inner) => Ok(SendDynError::Closed(
                *inner.downcast().map_err(SendDynError::Closed)?,
            )),
            Self::NotAccepted(inner) => Ok(SendDynError::NotAccepted(
                *inner.downcast().map_err(SendDynError::NotAccepted)?,
            )),
        }
    }
}

impl TrySendDynError<AnyBox> {
    pub fn downcast<M: Message + 'static>(
        self,
    ) -> Result<TrySendDynError<M>, TrySendDynError<AnyBox>> {
        match self {
            Self::Closed(inner) => Ok(TrySendDynError::Closed(
                *inner.downcast().map_err(TrySendDynError::Closed)?,
            )),
            Self::Full(inner) => Ok(TrySendDynError::Full(
                *inner.downcast().map_err(TrySendDynError::Full)?,
            )),
            Self::NotAccepted(inner) => Ok(TrySendDynError::NotAccepted(
                *inner.downcast().map_err(TrySendDynError::NotAccepted)?,
            )),
        }
    }
}
