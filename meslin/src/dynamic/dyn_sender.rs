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

/// A wrapper around a [`Box<dyn IsDynSender>`](IsDynSender) that allows for dynamic, type-checked sending.
///
/// Any sender can be converted into a dynamic sender, as long as the protocol it sends implements
/// [`trait@DynProtocol`] and [`AsSet`](type_sets::AsSet) (Derived using [`derive@DynProtocol`]).
/// This allows for type-checked conversions from a regular sender to a dynamic one, disallowing the
/// creation of [`struct@DynSender`]s that send messages not accepted by the protocol.
///
/// ## Sending
/// A [`struct@DynSender`] automatically implements [`Sends<M>`] for all messages `M` accepted by the
/// protocol. This means that you can just use a [`struct@DynSender`] instead of a statically typed sender in
/// most cases. If you need to send a message that is not accepted by the protocol, you can use the
/// `dyn_{...}`-send methods, which return an error if the message is not accepted.
///
/// ## Generics
/// The parameter `T` specifies the accepted messages of the dynamic sender. It can be specified
/// using the [`macro@Set`] or the [`macro@DynSender`] macros:
/// - `DynSender<Set![]>` == `DynSender![]`
/// - `DynSender<Set![Msg1, Msg2]>` == `DynSender![Msg1, Msg2]`
///
/// The parameter `W` specifies the type of the `with` parameter of the `send_with` methods.
///
/// ## Unchecked methods
/// The unchecked methods, **not** marked unsafe, allow creating [`DynSender`]s that send messages
/// not accepted by the protocol. Only use these methods if you are sure that the protocol accepts
/// the messages you send. If you are not sure, use the `try_transform` methods instead, which
/// return an error at runtime if the protocol does not accept the messages.
pub struct DynSender<T, W = ()> {
    sender: Box<dyn IsDynSender<With = W>>,
    t: PhantomData<fn() -> T>,
}

impl<T, W> DynSender<T, W> {
    /// Create a new `DynSender` from a statically typed sender.
    pub fn new<S>(sender: S) -> Self
    where
        S: IsStaticSender + IsDynSender<With = W>,
        T: SubsetOf<S::Protocol>,
    {
        Self::new_unchecked(sender)
    }

    /// Create a new `DynSender` from a statically typed sender, without checking if the protocol
    /// accepts the messages.
    pub fn new_unchecked<S>(sender: S) -> Self
    where
        S: IsDynSender<With = W>,
    {
        Self::from_inner_unchecked(Box::new(sender))
    }

    /// Transform the `DynSender` into one that accepts a subset of the messages.
    pub fn transform<R>(self) -> DynSender<R, W>
    where
        R: SubsetOf<T>,
    {
        DynSender::from_inner_unchecked(self.sender)
    }

    /// Attempt to transform the `DynSender` into a `DynSender` that accepts a subset of the messages,
    /// failing if the protocol does not accept the messages.
    pub fn try_transform<R>(self) -> Result<DynSender<R, W>, Self>
    where
        R: Members,
        W: 'static,
        T: 'static,
    {
        if R::members().iter().all(|t2| self.members().contains(t2)) {
            Ok(DynSender::from_inner_unchecked(self.sender))
        } else {
            Err(self)
        }
    }

    /// Transform the `DynSender` into a `DynSender` that accepts a subset of the messages, without
    /// checking if the protocol accepts the messages.
    pub fn transform_unchecked<R>(self) -> DynSender<R, W> {
        DynSender::from_inner_unchecked(self.sender)
    }

    pub fn try_from_inner(
        sender: Box<dyn IsDynSender<With = W>>,
    ) -> Result<Self, Box<dyn IsDynSender<With = W>>>
    where
        T: Members,
        W: 'static,
        T: 'static,
    {
        if T::members().iter().all(|t2| sender.members().contains(t2)) {
            Ok(Self::from_inner_unchecked(sender))
        } else {
            Err(sender)
        }
    }

    /// Convert a [`Box<dyn DynSends>`](DynSends) into a `DynSender`, without checking if the protocol
    /// accepts the messages.
    pub fn from_inner_unchecked(sender: Box<dyn IsDynSender<With = W>>) -> Self {
        Self {
            sender,
            t: PhantomData,
        }
    }

    /// Convert into a [`Box<dyn DynSends>`](DynSends).
    pub fn into_inner(self) -> Box<dyn IsDynSender<With = W>> {
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

impl<T, W> IsDynSender for DynSender<T, W>
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

    fn members(&self) -> &'static [TypeId] {
        self.sender.members()
    }

    fn clone_boxed(&self) -> Box<dyn IsDynSender<With = Self::With>> {
        self.sender.clone_boxed()
    }

    fn as_any(&self) -> &dyn Any {
        self.sender.as_any()
    }
}

impl<T, W, M> Sends<M> for DynSender<T, W>
where
    T: Contains<M>,
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

impl<T, W> Debug for DynSender<T, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynSender")
            .field("sender", &self.sender)
            .field("accepts", &type_name::<T>())
            .finish()
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
