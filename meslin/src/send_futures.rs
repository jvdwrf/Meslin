//! Future combinators for sending messages.
//!
//! This module contains combinators for sending messages, to be used with senders.
//! A send-future can be created with the [`IsSenderExt::send`] and [`IsSenderExt::send_msg`]
//! methods. These methods return a [`SendFut`] or [`SendMsgFut`] respectively.
//!
//! A send-future can then be altered, using the following modifiers:
//! - `with`: Provides a value to send the message with, instead of using the default.
//! - `recv`: After the message is sent, waits for a reply.
//! - `dynamic`: Sends the message dynamically, checking at runtime for acceptance.
//!
//! Once the send-future is ready, it can be executed with:
//! - `.await`: Waits for the message to be sent. (Uses [`IntoFuture`])
//! - `wait()`: Blocks the current thread until the message is sent.
//! - `now()`: Attempts to send the message without blocking. (Not available for requests)
//! 
//! # Example
//! ```
#![doc = include_str!("../examples/send_futures.rs")]
//! ```

use crate::*;
use futures::{executor::block_on, Future};
use std::future::IntoFuture;

pub use msg::*;
mod msg {
    use super::*;

    /// Sends a message with a given value.
    ///
    /// Can be executed with `.await`, `wait()` or `now()`.
    #[derive(derive_more::Debug)]
    pub struct SendMsgWithFut<'a, S: IsSender, M> {
        pub(super) inner: SendMsgFut<'a, S, M>,
        pub(super) with: S::With,
    }

    impl<'a, S: IsSender, M> SendMsgWithFut<'a, S, M> {
        /// Returns the sender, message and value.
        pub fn into_inner(self) -> (&'a S, M, S::With) {
            let (sender, msg) = self.inner.into_inner();
            (sender, msg, self.with)
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(self) -> Result<(), SendError<(M, S::With)>>
        where
            S: Sends<M>,
        {
            <S as Sends<M>>::send_msg_blocking_with(self.inner.sender, self.inner.msg, self.with)
        }

        /// Attempt to send the message without blocking.
        #[inline]
        pub fn now(self) -> Result<(), SendNowError<(M, S::With)>>
        where
            S: Sends<M>,
        {
            <S as Sends<M>>::send_msg_with_now(self.inner.sender, self.inner.msg, self.with)
        }

        /// Send the message dynamically, checking at runtime for acceptance.
        #[cfg(feature = "dynamic")]
        #[inline]
        pub fn dynamic(self) -> DynSendMsgWithFut<'a, S, M> {
            DynSendMsgWithFut(self)
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for SendMsgWithFut<'a, S, M>
    where
        S: Sends<M>,
    {
        type Output = Result<(), SendError<(M, S::With)>>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            <S as Sends<M>>::send_msg_with(self.inner.sender, self.inner.msg, self.with)
        }
    }

    /// Sends a message with a default value.
    #[derive(derive_more::Debug)]
    pub struct SendMsgFut<'a, S: IsSender, M> {
        pub(super) sender: &'a S,
        pub(super) msg: M,
    }

    impl<'a, S: IsSender, M> SendMsgFut<'a, S, M> {
        /// Returns the sender and message.
        pub fn into_inner(self) -> (&'a S, M) {
            (&self.sender, self.msg)
        }

        /// Create a new `SendMsgFut`.
        #[inline]
        pub fn new(sender: &'a S, msg: M) -> Self {
            Self { sender, msg }
        }

        /// Provide a value to send the message with, instead of using the default.
        #[inline]
        pub fn with(self, with: S::With) -> SendMsgWithFut<'a, S, M> {
            SendMsgWithFut { inner: self, with }
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(self) -> Result<(), SendError<M>>
        where
            S: Sends<M>,
            S::With: Default,
        {
            match self.with(Default::default()).wait() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(msg, _)| msg)),
            }
        }

        /// Attempt to send the message without blocking.
        #[inline]
        pub fn now(self) -> Result<(), SendNowError<M>>
        where
            S: Sends<M>,
            S::With: Default,
        {
            match self.with(Default::default()).now() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(msg, _)| msg)),
            }
        }

        /// Send the message dynamically, checking at runtime for acceptance.
        #[cfg(feature = "dynamic")]
        #[inline]
        pub fn dynamic(self) -> DynSendMsgFut<'a, S, M> {
            DynSendMsgFut(self)
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for SendMsgFut<'a, S, M>
    where
        S: Sends<M>,
        S::With: Default,
    {
        type Output = Result<(), SendError<M>>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = <S as Sends<M>>::send_msg_with(self.sender, self.msg, Default::default());
            async {
                match fut.await {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e.map(|(msg, _)| msg)),
                }
            }
        }
    }
}

pub use normal::*;
mod normal {
    use super::*;

    /// Sends a message with a given value.
    ///
    /// Can be executed with `.await`, `wait()` or `now()`.
    #[derive(derive_more::Debug)]
    pub struct SendWithFut<'a, S: IsSender, M: Message> {
        pub(super) inner: SendFut<'a, S, M>,
        pub(super) with: S::With,
    }

    impl<'a, S: IsSender, M: Message> SendWithFut<'a, S, M> {
        /// Returns the sender, input and value.
        pub fn into_inner(self) -> (&'a S, M::Input, S::With) {
            let (sender, input) = self.inner.into_inner();
            (sender, input, self.with)
        }

        #[inline]
        pub(super) fn with_msg(self) -> (SendMsgWithFut<'a, S, M>, M::Output) {
            let (msg, output) = M::create(self.inner.input);
            let combinator = SendMsgWithFut {
                inner: SendMsgFut {
                    sender: self.inner.sender,
                    msg,
                },
                with: self.with,
            };
            (combinator, output)
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(self) -> Result<M::Output, SendError<(M::Input, S::With)>>
        where
            S: Sends<M>,
        {
            let (combinator, output) = self.with_msg();

            match combinator.wait() {
                Ok(()) => Ok(output),
                Err(e) => Err(e.map(|(msg, with)| (msg.cancel(output), with))),
            }
        }

        /// Attempt to send the message without blocking.
        #[inline]
        pub fn now(self) -> Result<M::Output, SendNowError<(M::Input, S::With)>>
        where
            S: Sends<M>,
        {
            let (combinator, output) = self.with_msg();

            match combinator.now() {
                Ok(()) => Ok(output),
                Err(e) => Err(e.map(|(msg, with)| (msg.cancel(output), with))),
            }
        }

        /// After the message is sent, wait for a reply.
        #[inline]
        pub fn recv(self) -> RequestWithFut<'a, S, M> {
            RequestWithFut(self)
        }

        /// Send the message dynamically, checking at runtime for acceptance.
        #[cfg(feature = "dynamic")]
        #[inline]
        pub fn dynamic(self) -> DynSendWithFut<'a, S, M> {
            DynSendWithFut(self)
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for SendWithFut<'a, S, M>
    where
        S: Sends<M>,
    {
        type Output = Result<M::Output, SendError<(M::Input, S::With)>>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let (combinator, output) = self.with_msg();
            let fut = combinator.into_future();
            async {
                match fut.await {
                    Ok(()) => Ok(output),
                    Err(e) => Err(e.map(|(msg, with)| (msg.cancel(output), with))),
                }
            }
        }
    }

    /// Sends a message with a default value.
    ///
    /// Can be executed with `.await`, `wait()` or `now()`.
    #[derive(derive_more::Debug)]
    pub struct SendFut<'a, S: IsSender, M: Message> {
        pub(super) sender: &'a S,
        pub(super) input: M::Input,
    }

    impl<'a, S: IsSender, M: Message> SendFut<'a, S, M> {
        pub fn into_inner(self) -> (&'a S, M::Input) {
            (&self.sender, self.input)
        }

        /// Create a new `SendFut`.
        #[inline]
        pub fn new(sender: &'a S, input: M::Input) -> Self {
            Self { sender, input }
        }

        /// Provide a value to send the message with, instead of using the default.
        #[inline]
        pub fn with(self, with: S::With) -> SendWithFut<'a, S, M> {
            SendWithFut { inner: self, with }
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(self) -> Result<M::Output, SendError<M::Input>>
        where
            S: Sends<M>,
            S::With: Default,
        {
            match self.with(Default::default()).wait() {
                Ok(output) => Ok(output),
                Err(e) => Err(e.map(|(t, _)| t)),
            }
        }

        /// Attempt to send the message without blocking.
        #[inline]
        pub fn now(self) -> Result<M::Output, SendNowError<M::Input>>
        where
            S: Sends<M>,
            S::With: Default,
        {
            match self.with(Default::default()).now() {
                Ok(output) => Ok(output),
                Err(e) => Err(e.map(|(t, _)| t)),
            }
        }

        /// After the message is sent, wait for a reply.
        #[inline]
        pub fn recv(self) -> RequestFut<'a, S, M> {
            RequestFut(self)
        }

        /// Send the message dynamically, checking at runtime for acceptance.
        #[cfg(feature = "dynamic")]
        #[inline]
        pub fn dynamic(self) -> DynSendFut<'a, S, M> {
            DynSendFut(self)
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for SendFut<'a, S, M>
    where
        S: Sends<M>,
        S::With: Default,
    {
        type Output = Result<M::Output, SendError<M::Input>>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = self.with(Default::default()).into_future();
            async {
                match fut.await {
                    Ok(output) => Ok(output),
                    Err(e) => Err(e.map(|(t, _)| t)),
                }
            }
        }
    }
}

pub use request::*;
mod request {
    use super::*;

    /// Sends a message with a given value, and waits for a reply.
    ///
    /// Can be executed with `.await` or `wait()`.
    #[derive(derive_more::Debug)]
    pub struct RequestWithFut<'a, S: IsSender, M: Message>(pub(super) SendWithFut<'a, S, M>);

    impl<'a, S: IsSender, M: Message> RequestWithFut<'a, S, M> {
        /// Returns the sender, input and value.
        pub fn into_inner(self) -> (&'a S, M::Input, S::With) {
            self.0.into_inner()
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(
            self,
        ) -> Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<(M::Input, S::With), <M::Output as ResultFuture>::Error>,
        >
        where
            S: Sends<M>,
            M::Output: ResultFuture,
        {
            match block_on(self.0.wait()?) {
                Ok(val) => Ok(val),
                Err(e) => Err(RequestError::NoReply(e)),
            }
        }

        /// Send the message dynamically, checking at runtime for acceptance.
        #[cfg(feature = "dynamic")]
        #[inline]
        pub fn dynamic(self) -> DynRequestWithFut<'a, S, M> {
            DynRequestWithFut(DynSendWithFut(self.0))
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for RequestWithFut<'a, S, M>
    where
        S: Sends<M>,
        M::Output: ResultFuture,
        S::With: Send,
        M::Input: Send,
    {
        type Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<(M::Input, S::With), <M::Output as ResultFuture>::Error>,
        >;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = self.0.into_future();
            async {
                match fut.await?.await {
                    Ok(val) => Ok(val),
                    Err(e) => Err(RequestError::NoReply(e)),
                }
            }
        }
    }

    /// Sends a message with a default value, and waits for a reply.
    ///
    /// Can be executed with `.await` or `wait()`.
    #[derive(derive_more::Debug)]
    pub struct RequestFut<'a, S: IsSender, M: Message>(pub(super) SendFut<'a, S, M>);

    impl<'a, S: IsSender, M: Message> RequestFut<'a, S, M> {
        /// Returns the sender and input.
        pub fn into_inner(self) -> (&'a S, M::Input) {
            self.0.into_inner()
        }

        /// Provide a value to send the message with, instead of using the default.
        #[inline]
        pub fn with(self, with: S::With) -> RequestWithFut<'a, S, M> {
            RequestWithFut(self.0.with(with))
        }

        /// Block the current thread until the message is sent and the reply is received.
        ///
        /// Uses [`futures::executor::block_on`] to wait for the reply.
        #[inline]
        pub fn wait(
            self,
        ) -> Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<M::Input, <M::Output as ResultFuture>::Error>,
        >
        where
            S: Sends<M>,
            M::Output: ResultFuture,
            S::With: Default,
        {
            match block_on(self.0.wait()?) {
                Ok(val) => Ok(val),
                Err(e) => Err(RequestError::NoReply(e)),
            }
        }

        /// Send the message dynamically, checking at runtime for acceptance.
        #[cfg(feature = "dynamic")]
        #[inline]
        pub fn dynamic(self) -> DynRequestFut<'a, S, M> {
            DynRequestFut(DynSendFut(self.0))
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for RequestFut<'a, S, M>
    where
        S: Sends<M>,
        M::Output: ResultFuture,
        S::With: Default,
        M::Input: Send,
    {
        type Output = Result<
            <M::Output as ResultFuture>::Ok,
            RequestError<M::Input, <M::Output as ResultFuture>::Error>,
        >;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = self.0.into_future();
            async {
                match fut.await?.await {
                    Ok(val) => Ok(val),
                    Err(e) => Err(RequestError::NoReply(e)),
                }
            }
        }
    }
}

#[cfg(feature = "dynamic")]
pub use dynamic::*;
#[cfg(feature = "dynamic")]
mod dynamic {
    use super::*;

    /// Sends a message with a given value, checking at runtime for acceptance.
    ///
    /// Can be executed with `.await`, `wait()` or `now()`.
    #[derive(derive_more::Debug)]
    pub struct DynSendMsgWithFut<'a, S: IsSender, M>(pub(super) SendMsgWithFut<'a, S, M>);

    impl<'a, S: IsSender, M> DynSendMsgWithFut<'a, S, M> {
        /// Returns the sender, message and value.
        pub fn into_inner(self) -> (&'a S, M, S::With) {
            self.0.into_inner()
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(self) -> Result<(), DynSendError<(M, S::With)>>
        where
            S: IsDynSender,
            M: Send + 'static,
            S::With: Send + 'static,
        {
            <S as IsDynSender>::dyn_send_boxed_msg_blocking_with(
                self.0.inner.sender,
                BoxedMsg::new(self.0.inner.msg, self.0.with),
            )
            .map_err(|e| e.downcast::<M>().unwrap_silent())
        }

        /// Attempt to send the message without blocking.
        #[inline]
        pub fn now(self) -> Result<(), DynSendNowError<(M, S::With)>>
        where
            S: IsDynSender,
            M: Send + 'static,
            S::With: Send + 'static,
        {
            <S as IsDynSender>::dyn_try_send_boxed_msg_with(
                self.0.inner.sender,
                BoxedMsg::new(self.0.inner.msg, self.0.with),
            )
            .map_err(|e| e.downcast::<M>().unwrap_silent())
        }
    }

    impl<'a, S: IsSender, M> IntoFuture for DynSendMsgWithFut<'a, S, M>
    where
        S: IsDynSender,
        M: Send + 'static,
        S::With: Send + 'static,
    {
        type Output = Result<(), DynSendError<(M, S::With)>>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = <S as IsDynSender>::dyn_send_boxed_msg_with(
                self.0.inner.sender,
                BoxedMsg::new(self.0.inner.msg, self.0.with),
            );
            async { fut.await.map_err(|e| e.downcast::<M>().unwrap_silent()) }
        }
    }

    /// Sends a message with a default value, checking at runtime for acceptance.
    ///
    /// Can be executed with `.await`, `wait()` or `now()`.
    #[derive(derive_more::Debug)]
    pub struct DynSendMsgFut<'a, S: IsSender, M>(pub(super) SendMsgFut<'a, S, M>);

    impl<'a, S: IsSender, M> DynSendMsgFut<'a, S, M> {
        /// Returns the sender and message.
        pub fn into_inner(self) -> (&'a S, M) {
            self.0.into_inner()
        }

        /// Provide a value to send the message with, instead of using the default.
        #[inline]
        pub fn with(self, with: S::With) -> DynSendMsgWithFut<'a, S, M> {
            DynSendMsgWithFut(SendMsgWithFut {
                inner: self.0,
                with,
            })
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(self) -> Result<(), DynSendError<M>>
        where
            S: IsDynSender,
            M: Send + 'static,
            S::With: Default + Send + 'static,
        {
            self.with(Default::default())
                .wait()
                .map_err(|e| e.map(|(msg, _)| msg))
        }

        /// Attempt to send the message without blocking.
        #[inline]
        pub fn now(self) -> Result<(), DynSendNowError<M>>
        where
            S: IsDynSender,
            M: Send + 'static,
            S::With: Default + Send + 'static,
        {
            self.with(Default::default())
                .now()
                .map_err(|e| e.map(|(msg, _)| msg))
        }
    }

    impl<'a, S: IsSender, M> IntoFuture for DynSendMsgFut<'a, S, M>
    where
        S: IsDynSender,
        M: Send + 'static,
        S::With: Default + Send + 'static,
    {
        type Output = Result<(), DynSendError<M>>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = self.with(Default::default()).into_future();
            async { fut.await.map_err(|e| e.map(|(msg, _)| msg)) }
        }
    }

    /// Sends a message with a given value, and waits for a reply.
    ///
    /// Can be executed with `.await` or `wait()`.
    #[derive(derive_more::Debug)]
    pub struct DynSendWithFut<'a, S: IsSender, M: Message>(pub(super) SendWithFut<'a, S, M>);

    impl<'a, S: IsSender, M: Message> DynSendWithFut<'a, S, M> {
        /// Returns the sender, input and value.
        pub fn into_inner(self) -> (&'a S, M::Input, S::With) {
            self.0.into_inner()
        }

        #[inline]
        fn with_msg(self) -> (DynSendMsgWithFut<'a, S, M>, M::Output) {
            let (send_with_msg, output) = self.0.with_msg();
            (DynSendMsgWithFut(send_with_msg), output)
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(self) -> Result<M::Output, DynSendError<(M::Input, S::With)>>
        where
            S: IsDynSender,
            M: Send + 'static,
            S::With: Send + 'static,
        {
            let (send_with_msg, output) = self.with_msg();
            match send_with_msg.wait() {
                Ok(()) => Ok(output),
                Err(e) => Err(e.map(|(msg, with)| (msg.cancel(output), with))),
            }
        }

        /// Attempt to send the message without blocking.
        #[inline]
        pub fn now(self) -> Result<M::Output, DynSendNowError<(M::Input, S::With)>>
        where
            S: IsDynSender,
            M: Send + 'static,
            S::With: Send + 'static,
        {
            let (send_with_msg, output) = self.with_msg();
            match send_with_msg.now() {
                Ok(()) => Ok(output),
                Err(e) => Err(e.map(|(msg, with)| (msg.cancel(output), with))),
            }
        }

        /// After the message is sent, wait for a reply.
        #[inline]
        pub fn recv(self) -> DynRequestWithFut<'a, S, M> {
            DynRequestWithFut(self)
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for DynSendWithFut<'a, S, M>
    where
        S: IsDynSender,
        M: Send + 'static,
        S::With: Send + 'static,
    {
        type Output = Result<M::Output, DynSendError<(M::Input, S::With)>>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let (send_with_msg, output) = self.with_msg();
            let fut = send_with_msg.into_future();
            async {
                match fut.await {
                    Ok(()) => Ok(output),
                    Err(e) => Err(e.map(|(msg, with)| (msg.cancel(output), with))),
                }
            }
        }
    }

    /// Sends a message with a given value, checking at runtime for acceptance, and waits for a reply.
    ///
    /// Can be executed with `.await` or `wait()`.
    #[derive(derive_more::Debug)]
    pub struct DynRequestWithFut<'a, S: IsSender, M: Message>(pub(super) DynSendWithFut<'a, S, M>);

    impl<'a, S: IsSender, M: Message> DynRequestWithFut<'a, S, M> {
        /// Returns the sender, input and value.
        pub fn into_inner(self) -> (&'a S, M::Input, S::With) {
            self.0.into_inner()
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(
            self,
        ) -> Result<
            <M::Output as ResultFuture>::Ok,
            DynRequestError<(M::Input, S::With), <M::Output as ResultFuture>::Error>,
        >
        where
            S: IsDynSender,
            M: Send + 'static,
            M::Output: ResultFuture,
            S::With: Send + 'static,
        {
            match block_on(self.0.wait()?) {
                Ok(val) => Ok(val),
                Err(e) => Err(DynRequestError::NoReply(e)),
            }
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for DynRequestWithFut<'a, S, M>
    where
        S: IsDynSender,
        M: Send + 'static,
        M::Output: ResultFuture,
        S::With: Send + 'static,
        M::Input: Send,
    {
        type Output = Result<
            <M::Output as ResultFuture>::Ok,
            DynRequestError<(M::Input, S::With), <M::Output as ResultFuture>::Error>,
        >;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = self.0.into_future();
            async {
                match fut.await?.await {
                    Ok(val) => Ok(val),
                    Err(e) => Err(DynRequestError::NoReply(e)),
                }
            }
        }
    }

    /// Sends a message with a default value, checking at runtime for acceptance.
    ///
    /// Can be executed with `.await`, `wait()` or `now()`.
    #[derive(derive_more::Debug)]
    pub struct DynSendFut<'a, S: IsSender, M: Message>(pub(super) SendFut<'a, S, M>);

    impl<'a, S: IsSender, M: Message> DynSendFut<'a, S, M> {
        /// Returns the sender and input.
        pub fn into_inner(self) -> (&'a S, M::Input) {
            self.0.into_inner()
        }

        /// Provide a value to send the message with, instead of using the default.
        #[inline]
        pub fn with(self, with: S::With) -> DynSendWithFut<'a, S, M> {
            DynSendWithFut(SendWithFut {
                inner: self.0,
                with,
            })
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(self) -> Result<M::Output, DynSendError<M::Input>>
        where
            S: IsDynSender,
            M: Send + 'static,
            S::With: Default + Send + 'static,
        {
            self.with(Default::default())
                .wait()
                .map_err(|e| e.map(|(t, _)| t))
        }

        /// Attempt to send the message without blocking.
        #[inline]
        pub fn now(self) -> Result<M::Output, DynSendNowError<M::Input>>
        where
            S: IsDynSender,
            M: Send + 'static,
            S::With: Default + Send + 'static,
        {
            self.with(Default::default())
                .now()
                .map_err(|e| e.map(|(t, _)| t))
        }

        /// After the message is sent, wait for a reply.
        #[inline]
        pub fn recv(self) -> DynRequestFut<'a, S, M> {
            DynRequestFut(self)
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for DynSendFut<'a, S, M>
    where
        S: IsDynSender,
        M: Send + 'static,
        S::With: Default + Send + 'static,
    {
        type Output = Result<M::Output, DynSendError<M::Input>>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = self.with(Default::default()).into_future();
            async { fut.await.map_err(|e| e.map(|(t, _)| t)) }
        }
    }

    /// Sends a message with a default value, checking at runtime for acceptance, and waits for a reply.
    ///
    /// Can be executed with `.await` or `wait()`.
    #[derive(derive_more::Debug)]
    pub struct DynRequestFut<'a, S: IsSender, M: Message>(pub(super) DynSendFut<'a, S, M>);

    impl<'a, S: IsSender, M: Message> DynRequestFut<'a, S, M> {
        /// Returns the sender and input.
        pub fn into_inner(self) -> (&'a S, M::Input) {
            self.0.into_inner()
        }

        /// Block the current thread until the message is sent.
        #[inline]
        pub fn wait(
            self,
        ) -> Result<
            <M::Output as ResultFuture>::Ok,
            DynRequestError<M::Input, <M::Output as ResultFuture>::Error>,
        >
        where
            S: IsDynSender,
            M: Send + 'static,
            M::Output: ResultFuture,
            S::With: Default + Send + 'static,
        {
            match block_on(self.0.wait()?) {
                Ok(val) => Ok(val),
                Err(e) => Err(DynRequestError::NoReply(e)),
            }
        }

        /// Provide a value to send the message with, instead of using the default.
        #[inline]
        pub fn with(self, with: S::With) -> DynRequestWithFut<'a, S, M> {
            DynRequestWithFut(self.0.with(with))
        }
    }

    impl<'a, S: IsSender, M: Message> IntoFuture for DynRequestFut<'a, S, M>
    where
        S: IsDynSender,
        M: Send + 'static,
        M::Output: ResultFuture,
        S::With: Default + Send + 'static,
        M::Input: Send,
    {
        type Output = Result<
            <M::Output as ResultFuture>::Ok,
            DynRequestError<M::Input, <M::Output as ResultFuture>::Error>,
        >;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        #[inline]
        fn into_future(self) -> Self::IntoFuture {
            let fut = self.0.into_future();
            async {
                match fut.await?.await {
                    Ok(val) => Ok(val),
                    Err(e) => Err(DynRequestError::NoReply(e)),
                }
            }
        }
    }
}
