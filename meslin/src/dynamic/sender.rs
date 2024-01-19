use crate::*;
use ::type_sets::{Contains, Members, SubsetOf};
use futures::{future::BoxFuture, Future};
use std::{
    any::{type_name, Any, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

/// A macro that defines a [`struct@DynSender`].
///
/// Example:
/// - `DynSender![u32, u64]` == `DynSender<Set![u32, u64]>` == `DynSender<dyn Two<u32, u64>>`
/// - `DynSender![]` == `DynSender<Set![]>` == `DynSender<dyn Empty>`
/// - `DynSender![u32, u64; i32]` == `DynSender<Set![u32, u64], i32>` == `DynSender<dyn Two<u32, u64>, i32>`
#[macro_export]
macro_rules! DynSender {
    ($($msg:ty),* $(,)? $(; $with:ty)?) => {
        $crate::DynSender::<
            $crate::Set![$($msg),*], 
            $($with)?
        >
    };
}

/// A wrapper around a [`Box<dyn DynSends>`](DynSends) that allows for type-checked conversions.
///
/// Any sender can be converted into a `DynSender`, as long as the protocol it sends implements
/// [`trait@DynFromInto`] and marker traits [`AsSet`](type_sets::AsSet). This conversion is type-checked, so that
/// it is impossible to create [`struct@DynSender`]s that send messages not accepted by the protocol.
///
/// ## Sending
/// A [`struct@DynSender`] automatically implements [`Sends<M>`] for all messages `M` accepted by the
/// protocol. This means that you can just use a [`struct@DynSender`] instead of a statically typed sender in
/// most cases. If you need to send a message that is not accepted by the protocol, you can use the
/// `dyn_{...}`-send methods, which return an error if the message is not accepted.
///
/// ## Generics
/// The parameter `A` specifies the accepted messages of the dynamic sender. It can be specified
/// using the [`macro@Set`] or the [`macro@DynSender`] macros:
/// - `DynSender<Set![]>` == `DynSender![]`
/// - `DynSender<Set![Msg1, Msg2]>` == `DynSender![Msg1, Msg2]`
///
/// ## Unchecked methods
/// The unchecked methods, **not** marked unsafe, allow creating of `DynSender`s that send messages
/// not accepted by the protocol. Only use these methods if you are sure that the protocol accepts
/// the messages you send. If you are not sure, use the `try_transform` methods instead, which
/// return an error if the protocol does not accept the messages.
pub struct DynSender<A, W = ()> {
    sender: BoxedSender<W>,
    t: PhantomData<fn() -> A>,
}

impl<A, W> DynSender<A, W> {
    /// Create a new `DynSender` from a statically typed sender.
    pub fn new<S>(sender: S) -> Self
    where
        S: SendsProtocol + DynSends<With = W>,
        A: SubsetOf<S::Protocol>,
    {
        Self::new_unchecked(sender)
    }

    /// Create a new `DynSender` from a statically typed sender, without checking if the protocol
    /// accepts the messages.
    pub fn new_unchecked<S>(sender: S) -> Self
    where
        S: DynSends<With = W>,
    {
        Self::from_boxed_unchecked(Box::new(sender))
    }

    /// Transform the `DynSender` into a `DynSender` that accepts a subset of the messages.
    pub fn transform<A2>(self) -> DynSender<A2, W>
    where
        A2: SubsetOf<A>,
    {
        DynSender::from_boxed_unchecked(self.sender)
    }

    /// Attempt to transform the `DynSender` into a `DynSender` that accepts a subset of the messages,
    /// failing if the protocol does not accept the messages.
    pub fn try_transform<A2>(self) -> Result<DynSender<A2, W>, Self>
    where
        A2: Members,
        W: 'static,
        A: 'static,
    {
        if A2::members().iter().all(|t2| self.members().contains(t2)) {
            Ok(DynSender::from_boxed_unchecked(self.sender))
        } else {
            Err(self)
        }
    }

    /// Transform the `DynSender` into a `DynSender` that accepts a subset of the messages, without
    /// checking if the protocol accepts the messages.
    pub fn transform_unchecked<A2>(self) -> DynSender<A2, W> {
        DynSender::from_boxed_unchecked(self.sender)
    }

    /// Convert a [`Box<dyn DynSends>`](DynSends) into a `DynSender`, without checking if the protocol
    /// accepts the messages.
    pub fn from_boxed_unchecked(sender: BoxedSender<W>) -> Self {
        Self {
            sender,
            t: PhantomData,
        }
    }

    /// Convert into a [`Box<dyn DynSends>`](DynSends).
    pub fn into_boxed_sender(self) -> BoxedSender<W> {
        self.sender
    }

    /// Downcast the inner sender to a statically typed sender.
    pub fn downcast_ref<S>(&self) -> Option<&S>
    where
        S: IsSender<With = W> + 'static,
        W: 'static,
    {
        self.sender.as_any().downcast_ref::<S>()
    }
}

impl<A, W> IsSender for DynSender<A, W> {
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

impl<A, W> DynSends for DynSender<A, W>
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

    fn members(&self) -> &'static [TypeId] {
        self.sender.members()
    }

    fn clone_boxed(&self) -> BoxedSender<Self::With> {
        self.sender.clone_boxed()
    }

    fn as_any(&self) -> &dyn Any {
        self.sender.as_any()
    }
}

impl<A, W, M> Sends<M> for DynSender<A, W>
where
    A: Contains<M>,
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

impl<A, W: Debug> Debug for DynSender<A, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynSender")
            .field("sender", &"...")
            .field("t", &type_name::<A>())
            .finish()
    }
}

impl<A, W: 'static> Clone for DynSender<A, W> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            t: PhantomData,
        }
    }
}
