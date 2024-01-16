use std::any::TypeId;

use futures::Future;

use crate::*;

impl<T> DynSendsExt for T where T: DynSends {}
pub trait DynSendsExt: DynSends + Sized {
    fn into_boxed(self) -> BoxedSender<Self::With>
    where
        Self: Sized,
    {
        Box::new(self)
    }
    fn into_dyn<A: ?Sized>(self) -> DynSender<A, Self::With>
    where
        Self: SendsProtocol,
        A: TransformFrom<Self::Protocol>,
    {
        DynSender::new(self)
    }
    fn into_dyn_unchecked<A: ?Sized>(self) -> DynSender<A, Self::With>
    where
        Self: SendsProtocol,
    {
        DynSender::new_unchecked(self)
    }
    fn into_dyn_mapped<A: ?Sized, W>(self) -> DynSender<A, W>
    where
        Self: SendsProtocol + Sync + Clone,
        Self::With: Default,
        Self::Protocol: DynFromInto,
        W: Send + 'static,
        A: TransformFrom<Self::Protocol>,
    {
        DynSender::new_mapped(self)
    }
    fn into_dyn_mapped_unchecked<A: ?Sized, W>(self) -> DynSender<A, W>
    where
        Self: SendsProtocol + Sync + Clone,
        Self::With: Default,
        Self::Protocol: DynFromInto,
        W: Send + 'static,
    {
        DynSender::new_mapped_unchecked(self)
    }

    /// See [`SendsExt::send_msg_with`].
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
    fn dyn_send_msg<M>(&self, msg: M) -> impl Future<Output = Result<(), DynSendError<M>>> + Send
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
    {
        let fut = self.dyn_send_msg_with(msg, Default::default());
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map_first()),
            }
        }
    }
    fn dyn_send_msg_blocking<M>(&self, msg: M) -> Result<(), DynSendError<M>>
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
    {
        match self.dyn_send_msg_blocking_with(msg, Default::default()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map_first()),
        }
    }
    fn dyn_try_send_msg<M>(&self, msg: M) -> Result<(), DynTrySendError<M>>
    where
        M: Send + 'static,
        Self::With: Default + Send + 'static,
    {
        match self.dyn_try_send_msg_with(msg, Default::default()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map_first()),
        }
    }

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
                Err(e) => Err(e.map_cancel_first(output)),
            }
        }
    }
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
            Err(e) => Err(e.map_cancel_first(output)),
        }
    }
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
            Err(e) => Err(e.map_cancel_first(output)),
        }
    }
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
                Err(e) => Err(e.map_first()),
            }
        }
    }
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
            Err(e) => Err(e.map_first()),
        }
    }
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
            Err(e) => Err(e.map_first()),
        }
    }

    fn accepts_msg(&self, msg_id: TypeId) -> bool {
        self.accepts_all().contains(&msg_id)
    }
}
