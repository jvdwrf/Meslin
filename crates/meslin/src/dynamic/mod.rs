use crate::*;
use futures::{future::BoxFuture, Future};
use std::{
    any::{type_name, TypeId},
    marker::PhantomData,
};
mod dyn_sends_ext;
mod wrappers;
pub use dyn_sends_ext::*;
mod dyn_sends;
pub use dyn_sends::*;

/// DynSender<Accepts![Ping, Pong], u32>
/// DynSender<NoClone<AcceptTwo<Ping, Pong>>, u32>
pub struct DynSender<T, W = ()> {
    sender: Box<dyn DynSends<With = W>>,
    t: PhantomData<fn() -> T>,
}

/// A marker trait for [`AcceptsDyn`], to signal that a message is accepted.
///
/// When implemented on a type that is not actually accepted, the `send`
/// methods will panic.
///
/// This can be derived on an enum using [`macro@AcceptsDyn`]
pub trait Accepts<M, W = ()> {}

impl<T, W> DynSender<T, W> {
    pub fn new_unchecked<S>(sender: S) -> Self
    where
        S: DynSends<With = W>,
    {
        Self {
            sender: Box::new(sender),
            t: PhantomData,
        }
    }

    pub fn from_inner_unchecked(sender: Box<dyn DynSends<With = W>>) -> Self {
        Self {
            sender,
            t: PhantomData,
        }
    }

    pub fn into_inner(self) -> Box<dyn DynSends<With = W>> {
        self.sender
    }
}

impl<T, W: 'static> Clone for DynSender<T, W> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            t: PhantomData,
        }
    }
}

impl<T, W> IsSender for DynSender<T, W> {
    type With = W;

    fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    fn capacity(&self) -> Option<usize> {
        self.sender.capacity()
    }

    fn len(&self) -> usize {
        self.sender.len()
    }

    fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    fn sender_count(&self) -> usize {
        self.sender.sender_count()
    }
}

impl<T, W> DynSends for DynSender<T, W>
where
    T: 'static,
    W: 'static,
{
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>> {
        self.sender.dyn_send_boxed_msg_with(msg)
    }

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>> {
        self.sender.dyn_send_boxed_msg_blocking_with(msg)
    }

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynTrySendError<BoxedMsg<Self::With>>> {
        self.sender.dyn_try_send_boxed_msg_with(msg)
    }

    fn accepts_all(&self) -> &'static [TypeId] {
        self.sender.accepts_all()
    }

    fn clone_boxed(&self) -> Box<dyn DynSends<With = Self::With>> {
        self.sender.clone_boxed()
    }
}

impl<T, W, M> Sends<M> for DynSender<T, W>
where
    T: Accepts<M, W>,
    M: Send + 'static,
    W: Send + 'static,
{
    fn send_msg_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> impl Future<Output = Result<(), SendError<(M, Self::With)>>> + Send {
        let fut = this.sender.dyn_send_msg_with(msg, with);
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(match e {
                    DynSendError::NotAccepted(_e) => {
                        panic!("Message not accepted: {}", type_name::<(M, Self::With)>())
                    }
                    DynSendError::Closed((msg, with)) => SendError((msg, with)),
                }),
            }
        }
    }

    fn try_send_msg_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> Result<(), TrySendError<(M, Self::With)>> {
        match this.sender.dyn_try_send_msg_with(msg, with) {
            Ok(()) => Ok(()),
            Err(e) => Err(match e {
                DynTrySendError::NotAccepted(_e) => {
                    panic!("Message not accepted: {}", type_name::<(M, Self::With)>())
                }
                DynTrySendError::Closed((msg, with)) => TrySendError::Closed((msg, with)),
                DynTrySendError::Full((msg, with)) => TrySendError::Full((msg, with)),
            }),
        }
    }
}
