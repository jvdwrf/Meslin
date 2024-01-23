use crate::*;
use ::type_sets::Members;
use futures::{future::BoxFuture, Future};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

/// Automatically implemented when [`IsStaticSender`] is implemented for a protocol
/// that implements [`DynProtocol`].
pub trait IsDynSender: IsSender + Send + 'static + Debug {
    #[doc(hidden)]
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>>;

    #[doc(hidden)]
    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>>;

    #[doc(hidden)]
    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendNowError<BoxedMsg<Self::With>>>;

    /// Get the message types that the sender accepts.
    fn accepts_messages(&self) -> Vec<TypeId>;
    #[doc(hidden)]
    fn clone_boxed(&self) -> Box<dyn IsDynSender<With = Self::With>>;
    #[doc(hidden)]
    fn as_any(&self) -> &dyn Any;

    #[doc(hidden)]
    /// Like [`SendsExt::send_msg_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_msg_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> impl Future<Output = Result<(), DynSendError<(M, Self::With)>>> + Send
    where
        M: Send + 'static,
        Self::With: Send + 'static,
        Self: Sized,
    {
        let fut = self.dyn_send_boxed_msg_with(BoxedMsg::new(msg, with));
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.downcast::<M>().unwrap_silent()),
            }
        }
    }

    #[doc(hidden)]
    /// Like [`SendsExt::send_msg_blocking_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_msg_blocking_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> Result<(), DynSendError<(M, Self::With)>>
    where
        M: Send + 'static,
        Self::With: Send + 'static,
        Self: Sized,
    {
        match self.dyn_send_boxed_msg_blocking_with(BoxedMsg::new(msg, with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.downcast::<M>().unwrap_silent()),
        }
    }

    #[doc(hidden)]
    /// Like [`SendsExt::try_send_msg_with`], but fails if the message is not accepted by the protocol.
    fn dyn_try_send_msg_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> Result<(), DynSendNowError<(M, Self::With)>>
    where
        M: Send + 'static,
        Self::With: Send + 'static,
        Self: Sized,
    {
        match self.dyn_try_send_boxed_msg_with(BoxedMsg::new(msg, with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.downcast::<M>().unwrap()),
        }
    }
}

impl<T> IsDynSender for T
where
    T: IsStaticSender + Clone + Send + Sync + 'static + Debug,
    T::Protocol: DynProtocol,
    T::With: Send,
{
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>> {
        Box::pin(async move {
            let (protocol, with) = <T::Protocol as DynProtocol>::try_from_boxed_msg(msg)
                .map_err(DynSendError::NotAccepted)?;

            T::send_protocol_with(self, protocol, with).await.map_err(
                |SendError((protocol, with))| DynSendError::Closed(protocol.into_boxed_msg(with)),
            )
        })
    }

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>> {
        let (protocol, with) =
            T::Protocol::try_from_boxed_msg(msg).map_err(DynSendError::NotAccepted)?;

        T::send_protocol_blocking_with(self, protocol, with).map_err(
            |SendError((protocol, with))| DynSendError::Closed(protocol.into_boxed_msg(with)),
        )
    }

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendNowError<BoxedMsg<Self::With>>> {
        let (protocol, with) =
            T::Protocol::try_from_boxed_msg(msg).map_err(DynSendNowError::NotAccepted)?;

        T::try_send_protocol_with(self, protocol, with).map_err(|e| match e {
            SendNowError::Closed((protocol, with)) => {
                DynSendNowError::Closed(protocol.into_boxed_msg(with))
            }
            SendNowError::Full((protocol, with)) => {
                DynSendNowError::Full(protocol.into_boxed_msg(with))
            }
        })
    }

    /// Get a list of messages that can be sent to the underlying sender.
    fn accepts_messages(&self) -> Vec<TypeId> {
        <T::Protocol as Members>::members()
    }

    fn clone_boxed(&self) -> Box<dyn IsDynSender<With = Self::With>> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<W> IsSender for Box<dyn IsDynSender<With = W>> {
    type With = W;

    fn is_closed(&self) -> bool {
        (**self).is_closed()
    }

    fn capacity(&self) -> Option<usize> {
        (**self).capacity()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn receiver_count(&self) -> usize {
        (**self).receiver_count()
    }

    fn sender_count(&self) -> usize {
        (**self).sender_count()
    }
}

impl<W: 'static> IsDynSender for Box<dyn IsDynSender<With = W>> {
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>> {
        (**self).dyn_send_boxed_msg_with(msg)
    }

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>> {
        (**self).dyn_send_boxed_msg_blocking_with(msg)
    }

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendNowError<BoxedMsg<Self::With>>> {
        (**self).dyn_try_send_boxed_msg_with(msg)
    }

    fn accepts_messages(&self) -> Vec<TypeId> {
        (**self).accepts_messages()
    }

    #[doc(hidden)]
    fn clone_boxed(&self) -> Box<dyn IsDynSender<With = Self::With>> {
        (**self).clone_boxed()
    }

    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }
}

impl<T: 'static> Clone for Box<dyn IsDynSender<With = T>> {
    fn clone(&self) -> Self {
        (**self).clone_boxed()
    }
}

impl<W, T> From<T> for Box<dyn IsDynSender<With = W>>
where
    T: IsStaticSender<With = W> + Clone + Send + Sync + 'static + Debug,
    T::Protocol: DynProtocol,
    W: Send + 'static,
{
    fn from(sender: T) -> Self {
        Box::new(sender)
    }
}

impl<W, T> From<DynSender<T, W>> for Box<dyn IsDynSender<With = W>> {
    fn from(sender: DynSender<T, W>) -> Self {
        sender.into_inner()
    }
}

/// Extension trait for [`IsDynSender`], providing methods for dynamic dispatch.
///
/// This trait is automatically implemented for any senders that send a protocol which
/// implements [`DynProtocol`]. It is also implemented for `Box<dyn DynSends>` and [`struct@DynSender`].
pub trait IsDynSenderExt: IsDynSender + Sized {
    /// Check if the sender accepts a message.
    fn accepts<M: 'static>(&self) -> bool {
        self.accepts_messages().contains(&TypeId::of::<M>())
    }

    /// Convert the sender into a boxed sender.
    fn boxed(self) -> Box<dyn IsDynSender<With = Self::With>> {
        Box::new(self)
    }
}
impl<T> IsDynSenderExt for T where T: IsDynSender {}
