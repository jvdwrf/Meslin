use crate::{
    dynamic::DynSpec,
    message::{Message, Protocol},
    specification::{AddressSpec, DynAddressSpec, IntoSpec, SendDynError, SendError, StateSpec},
};
use futures::executor::block_on;
use std::{
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Clone, Debug)]
pub struct Address<S = DynSpec> {
    t: PhantomData<S>,
    spec: S,
}

impl<S: AddressSpec> Unpin for Address<S> {}

impl<S: AddressSpec + Unpin> Future for Address<S> {
    type Output = S::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<S::Output> {
        let inner = &mut self.spec;
        S::poll_address(Pin::new(inner), cx)
    }
}

impl<S> Address<S> {
    pub(crate) fn from_inner(inner: S) -> Self {
        Self {
            t: PhantomData,
            spec: inner,
        }
    }
}

#[derive(Debug)]
pub enum RequestError<T, R> {
    Send(SendError<T>),
    Recv(R),
}

#[derive(Debug)]
pub enum RequestDynError<T, R> {
    Send(SendDynError<T>),
    Recv(R),
}

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

impl<S: AddressSpec> Address<S> {
    pub fn into_spec<T>(self) -> Address<T>
    where
        S: IntoSpec<T>,
    {
        Address {
            spec: self.spec.into_spec(),
            t: PhantomData,
        }
    }

    pub fn wait(&mut self) -> S::Output
    where
        S: Unpin,
    {
        block_on(self)
    }

    pub fn is_alive(&self) -> bool {
        S::is_alive(&self.spec)
    }

    pub async fn send<M>(&self, msg: impl Into<M::Input>) -> Result<M::Output, SendError<M::Input>>
    where
        M: Message,
        S::Protocol: Protocol<M>,
    {
        let (msg, returned) = M::create(msg.into());
        match self
            .spec
            .send_protocol(<S::Protocol as Protocol<M>>::from_msg(msg))
            .await
        {
            Ok(()) => Ok(returned),
            Err(e) => Err(e.cancel_protocol::<M>(returned)),
        }
    }

    pub async fn request<M>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestError<M::Input, <M::Output as ResultFuture>::Error>,
    >
    where
        M: Message,
        S::Protocol: Protocol<M>,
        M::Output: ResultFuture + Send + 'static,
    {
        let rx = self.send(msg).await.map_err(RequestError::Send)?;
        match rx.await {
            Ok(b) => Ok(b),
            Err(a) => Err(RequestError::Recv(a)),
        }
    }

    pub async fn send_dyn<M>(
        &self,
        msg: impl Into<M::Input>,
    ) -> Result<M::Output, SendDynError<M::Input>>
    where
        M: Message + Send + 'static,
        S: DynAddressSpec,
    {
        let (payload, output) = M::create(msg.into());
        match self.spec.send_msg_dyn(payload).await {
            Ok(()) => Ok(output),
            Err(e) => Err(e.cancel(output)),
        }
    }

    pub async fn request_dyn<M>(
        &self,
        msg: impl Into<M::Input> + Send + 'static,
    ) -> Result<
        <M::Output as ResultFuture>::Ok,
        RequestDynError<M::Input, <M::Output as ResultFuture>::Error>,
    >
    where
        S: DynAddressSpec,
        M: Message + Send + 'static,
        M::Output: ResultFuture + Send + 'static,
    {
        let (payload, rx) = M::create(msg.into());
        match self.spec.send_msg_dyn(payload).await {
            Ok(()) => match rx.await {
                Ok(b) => Ok(b),
                Err(a) => Err(RequestDynError::Recv(a)),
            },
            Err(e) => Err(RequestDynError::Send(e.cancel(rx))),
        }
    }

    /// The shared state of the process.
    pub fn state(&self) -> &S::State
    where
        S: StateSpec,
    {
        S::state(&self.spec)
    }
}
