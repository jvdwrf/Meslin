use crate::*;
use futures::{future::BoxFuture, Future};
use std::{
    any::{type_name, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

use super::wrappers::MappedWithSender;

pub struct DynSender<A: ?Sized, W = ()> {
    sender: BoxedSender<W>,
    t: PhantomData<fn() -> A>,
}

impl<A: ?Sized, W> DynSender<A, W> {
    pub fn new<S>(sender: S) -> Self
    where
        S: SendsProtocol + DynSends<With = W>,
        A: TransformFrom<S::Protocol>,
    {
        Self::new_unchecked(sender)
    }

    pub fn new_mapped<S>(sender: S) -> Self
    where
        S: SendsProtocol + DynSends + Sync + Clone,
        S::With: Default,
        S::Protocol: DynFromInto,
        W: Send + 'static,
        A: TransformFrom<S::Protocol>,
    {
        Self::new_mapped_unchecked(sender)
    }

    pub fn new_unchecked<S>(sender: S) -> Self
    where
        S: DynSends<With = W>,
    {
        Self::from_boxed_unchecked(Box::new(sender))
    }

    pub fn new_mapped_unchecked<S>(sender: S) -> Self
    where
        S: SendsProtocol + DynSends + Sync + Clone,
        S::With: Default,
        S::Protocol: DynFromInto,
        W: Send + 'static,
    {
        let mapped_sender = MappedWithSender::<_, W>::new(sender);
        Self::new_unchecked(mapped_sender)
    }

    pub fn transform<A2: ?Sized>(self) -> DynSender<A2, W>
    where
        A2: TransformFrom<A>,
    {
        DynSender::from_boxed_unchecked(self.sender)
    }

    pub fn try_transform<A2: ?Sized>(self) -> Result<DynSender<A2, W>, Self>
    where
        A2: AcceptsAll,
        W: 'static,
        A: 'static,
    {
        if A2::accepts_all()
            .iter()
            .all(|t2| self.accepts_all().contains(t2))
        {
            Ok(DynSender::from_boxed_unchecked(self.sender))
        } else {
            Err(self)
        }
    }

    pub fn transform_unchecked<T2: ?Sized>(self) -> DynSender<T2, W> {
        DynSender::from_boxed_unchecked(self.sender)
    }

    pub fn from_boxed_unchecked(sender: BoxedSender<W>) -> Self {
        Self {
            sender,
            t: PhantomData,
        }
    }

    pub fn into_boxed_sender(self) -> BoxedSender<W> {
        self.sender
    }
}

impl<A: ?Sized, W> IsSender for DynSender<A, W> {
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

impl<A: ?Sized, W> DynSends for DynSender<A, W>
where
    A: 'static,
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

    fn clone_boxed(&self) -> BoxedSender<Self::With> {
        self.sender.clone_boxed()
    }
}

impl<A: ?Sized, W, M> Sends<M> for DynSender<A, W>
where
    A: Accepts<M>,
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

impl<A: ?Sized, W: Debug> Debug for DynSender<A, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynSender")
            .field("sender", &"...")
            .field("t", &type_name::<A>())
            .finish()
    }
}

impl<A: ?Sized, W: 'static> Clone for DynSender<A, W> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            t: PhantomData,
        }
    }
}
