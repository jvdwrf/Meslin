use std::any::TypeId;

use futures::Future;
use ::type_sets::SubsetOf;

use crate::*;

/// Extension trait for [`DynSends`], providing methods for dynamic dispatch.
///
/// This trait is automatically implemented for any senders that send a protocol which
/// implements [`DynFromInto`]. It is also implemented for `Box<dyn DynSends>` and [`struct@DynSender`].
pub trait DynSendsExt: DynSends + Sized {
    /// Check if the sender accepts a message.
    fn accepts(&self, msg_id: TypeId) -> bool {
        self.members().contains(&msg_id)
    }

    /// Convert the sender into a boxed sender.
    fn into_boxed(self) -> BoxedSender<Self::With>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    /// Convert the sender into a [`struct@DynSender`].
    fn into_dyn<A>(self) -> DynSender<A, Self::With>
    where
        Self: SendsProtocol,
        A: SubsetOf<Self::Protocol>,
    {
        DynSender::new(self)
    }

    /// Convert the sender into a [`struct@DynSender`], without checking if the protocol accepts the messages.
    fn into_dyn_unchecked<A>(self) -> DynSender<A, Self::With>
    where
        Self: SendsProtocol,
    {
        DynSender::new_unchecked(self)
    }

    /// Map the `with` value of the sender to `()`, by providing the default `with` to use.
    fn with(self, with: Self::With) -> WithValueSender<Self>
    where
        Self: SendsProtocol,
        Self::With: Clone,
    {
        WithValueSender::new(self, with)
    }

    /// Map the `with` value of the sender to `W`, by providing conversion functions.
    fn map_with<W>(
        self,
        f1: fn(W) -> Self::With,
        f2: fn(Self::With) -> W,
    ) -> MappedWithSender<Self, W>
    where
        Self: SendsProtocol + Send + Sync,
    {
        MappedWithSender::new(self, f1, f2)
    }

    /// Like [`SendsExt::send_msg_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_msg_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> impl Future<Output = Result<(), DynSendError<(M, Self::With)>>> + Send
    where
        M: Send + 'static,
        Self::With: Send + 'static,
    {
        let fut = self.dyn_send_boxed_msg_with(BoxedMsg::new(msg, with));
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.downcast::<M>().unwrap_silent()),
            }
        }
    }

    /// Like [`SendsExt::send_msg_blocking_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_msg_blocking_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> Result<(), DynSendError<(M, Self::With)>>
    where
        M: Send + 'static,
        Self::With: Send + 'static,
    {
        match self.dyn_send_boxed_msg_blocking_with(BoxedMsg::new(msg, with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.downcast::<M>().unwrap_silent()),
        }
    }

    /// Like [`SendsExt::try_send_msg_with`], but fails if the message is not accepted by the protocol.
    fn dyn_try_send_msg_with<M>(
        &self,
        msg: M,
        with: Self::With,
    ) -> Result<(), DynTrySendError<(M, Self::With)>>
    where
        M: Send + 'static,
        Self::With: Send + 'static,
    {
        match self.dyn_try_send_boxed_msg_with(BoxedMsg::new(msg, with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.downcast::<M>().unwrap()),
        }
    }

    /// Like [`SendsExt::send_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_msg<M>(&self, msg: M) -> impl Future<Output = Result<(), DynSendError<M>>> + Send
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
    {
        let fut = self.dyn_send_msg_with(msg, Default::default());
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(t, _)| t)),
            }
        }
    }

    /// Like [`SendsExt::send_blocking_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_msg_blocking<M>(&self, msg: M) -> Result<(), DynSendError<M>>
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
    {
        match self.dyn_send_msg_blocking_with(msg, Default::default()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(t, _)| t)),
        }
    }

    /// Like [`SendsExt::try_send_msg_with`], but fails if the message is not accepted by the protocol.
    fn dyn_try_send_msg<M>(&self, msg: M) -> Result<(), DynTrySendError<M>>
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
    {
        match self.dyn_try_send_msg_with(msg, Default::default()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(t, _)| t)),
        }
    }

    /// Like [`SendsExt::send_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: Self::With,
    ) -> impl Future<Output = Result<M::Output, DynSendError<(M::Input, Self::With)>>> + Send
    where
        M: Send + 'static,
        Self::With: Send + 'static,
        M::Output: Send,
    {
        let (msg, output) = M::create(msg.into());
        let fut = self.dyn_send_msg_with(msg, with);
        async {
            match fut.await {
                Ok(()) => Ok(output),
                Err(e) => Err(e.map(|(t, w)| (t.cancel(output), w))),
            }
        }
    }

    /// Like [`SendsExt::send_blocking_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_blocking_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: Self::With,
    ) -> Result<M::Output, DynSendError<(M::Input, Self::With)>>
    where
        M: Send + 'static,
        Self::With: Send + 'static,
        M::Output: Send,
    {
        let (msg, output) = M::create(msg.into());
        match self.dyn_send_msg_blocking_with(msg, with) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.map(|(t, w)| (t.cancel(output), w))),
        }
    }

    /// Like [`SendsExt::try_send_with`], but fails if the message is not accepted by the protocol.
    fn dyn_try_send_with<M: Message>(
        &self,
        msg: impl Into<M::Input>,
        with: Self::With,
    ) -> Result<M::Output, DynTrySendError<(M::Input, Self::With)>>
    where
        M: Send + 'static,
        Self::With: Send + 'static,
        M::Output: Send,
    {
        let (msg, output) = M::create(msg.into());
        match self.dyn_try_send_msg_with(msg, with) {
            Ok(()) => Ok(output),
            Err(e) => Err(e.map(|(t, w)| (t.cancel(output), w))),
        }
    }

    /// Like [`SendsExt::send_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> impl Future<Output = Result<M::Output, DynSendError<M::Input>>> + Send
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
        M::Output: Send,
    {
        let fut = self.dyn_send_with::<M>(msg, Default::default());
        async {
            match fut.await {
                Ok(output) => Ok(output),
                Err(e) => Err(e.map(|(t, _)| t)),
            }
        }
    }

    /// Like [`SendsExt::send_blocking_with`], but fails if the message is not accepted by the protocol.
    fn dyn_send_blocking<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, DynSendError<M::Input>>
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
        M::Output: Send,
    {
        match self.dyn_send_blocking_with::<M>(msg, Default::default()) {
            Ok(output) => Ok(output),
            Err(e) => Err(e.map(|(t, _)| t)),
        }
    }

    /// Like [`SendsExt::try_send_with`], but fails if the message is not accepted by the protocol.
    fn dyn_try_send<M: Message>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, DynTrySendError<M::Input>>
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
        M::Output: Send,
    {
        match self.dyn_try_send_with::<M>(msg, Default::default()) {
            Ok(output) => Ok(output),
            Err(e) => Err(e.map(|(t, _)| t)),
        }
    }
}
impl<T> DynSendsExt for T where T: DynSends {}
