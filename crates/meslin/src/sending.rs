use crate::*;
use std::future::Future;

//-------------------------------------
// Implemented by the user
//-------------------------------------

#[allow(clippy::len_without_is_empty)]
pub trait IsSender {
    type With;

    fn is_closed(&self) -> bool;
    fn capacity(&self) -> Option<usize>;
    fn len(&self) -> usize;
    fn receiver_count(&self) -> usize;
    fn sender_count(&self) -> usize;
}

/// Send a message, and wait for space
pub trait SendsProtocol: IsSender {
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

//-------------------------------------
// Automatically implemented
//-------------------------------------

/// Automatically implemented when [`SendProtocol`] is implemented.
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
    T: SendsProtocol + Send + Sync,
    T::Protocol: From<M> + TryInto<M>,
    M: Send,
    T::With: Send,
{
    async fn send_msg_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> Result<(), SendError<(M, Self::With)>> {
        T::send_protocol_with(this, T::Protocol::from(msg), with)
            .await
            .map_err(|e| e.map_into_msg_unwrap())
    }

    fn send_msg_blocking_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> Result<(), SendError<(M, Self::With)>> {
        T::send_protocol_blocking_with(this, T::Protocol::from(msg), with)
            .map_err(|e| e.map_into_msg_unwrap())
    }

    fn try_send_msg_with(
        this: &Self,
        msg: M,
        with: Self::With,
    ) -> Result<(), TrySendError<(M, Self::With)>> {
        T::try_send_protocol_with(this, T::Protocol::from(msg), with)
            .map_err(|e| e.map_into_msg_first_unwrap())
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
        async { fut.await.map_err(|e| e.map_first()) }
    }
    fn send_msg_blocking<M: Message>(&self, msg: M) -> Result<(), SendError<M>>
    where
        Self: Sends<M>,
        Self::With: Default,
    {
        self.send_msg_blocking_with(msg, Default::default())
            .map_err(|e| e.map_first())
    }
    fn try_send_msg<M: Message>(&self, msg: M) -> Result<(), TrySendError<M>>
    where
        Self: Sends<M>,
        Self::With: Default,
    {
        self.try_send_msg_with(msg, Default::default())
            .map_err(|e| e.map_into_first())
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
                Err(e) => Err(e.map_cancel_first(output)),
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
            Err(e) => Err(e.map_cancel_first(output)),
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
            Err(e) => Err(e.map_cancel_first(output)),
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
        async { fut.await.map_err(|e| e.map_first()) }
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
            .map_err(|e| e.map_first())
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
            .map_err(|e| e.map_into_first())
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
