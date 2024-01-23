use crate::*;
use futures::{executor::block_on, Future};
use std::future::IntoFuture;
use derive_more::Debug;

mod impl_traits;

//-------------------------------------
// SendMsgWith
//-------------------------------------

#[derive(Debug)]
pub struct SendMsgWithFut<'a, S: IsSender, M> {
    inner: SendMsgFut<'a, S, M>,
    with: S::With,
}

impl<'a, S: IsSender, M> SendMsgWithFut<'a, S, M> {
    #[inline]
    pub fn wait(self) -> Result<(), SendError<(M, S::With)>>
    where
        S: Sends<M>,
    {
        <S as Sends<M>>::send_msg_blocking_with(self.inner.sender, self.inner.msg, self.with)
    }

    #[inline]
    pub fn now(self) -> Result<(), SendNowError<(M, S::With)>>
    where
        S: Sends<M>,
    {
        <S as Sends<M>>::try_send_msg_with(self.inner.sender, self.inner.msg, self.with)
    }

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

//-------------------------------------
// SendMsg
//-------------------------------------

#[derive(Debug)]
pub struct SendMsgFut<'a, S: IsSender, M> {
    sender: &'a S,
    msg: M,
}

impl<'a, S: IsSender, M> SendMsgFut<'a, S, M> {
    #[inline]
    pub fn new(sender: &'a S, msg: M) -> Self {
        Self { sender, msg }
    }

    #[inline]
    pub fn with(self, with: S::With) -> SendMsgWithFut<'a, S, M> {
        SendMsgWithFut { inner: self, with }
    }

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

    #[inline]
    pub fn dynamic(self) -> SendDynMsgFut<'a, S, M> {
        SendDynMsgFut(self)
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

//-------------------------------------
// SendWith
//-------------------------------------

#[derive(Debug)]
pub struct SendWithFut<'a, S: IsSender, M: Message> {
    inner: SendFut<'a, S, M>,
    with: S::With,
}

impl<'a, S: IsSender, M: Message> SendWithFut<'a, S, M> {
    #[inline]
    fn with_msg(self) -> (SendMsgWithFut<'a, S, M>, M::Output) {
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

    #[inline]
    pub fn recv(self) -> RequestWithFut<'a, S, M> {
        RequestWithFut(self)
    }

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

//-------------------------------------
// RequestWith
//-------------------------------------

#[derive(Debug)]
pub struct RequestWithFut<'a, S: IsSender, M: Message>(SendWithFut<'a, S, M>);

impl<'a, S: IsSender, M: Message> RequestWithFut<'a, S, M> {
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

//-------------------------------------
// Send
//-------------------------------------

#[derive(Debug)]
pub struct SendFut<'a, S: IsSender, M: Message> {
    sender: &'a S,
    input: M::Input,
}

impl<'a, S: IsSender, M: Message> SendFut<'a, S, M> {
    #[inline]
    pub fn new(sender: &'a S, input: M::Input) -> Self {
        Self { sender, input }
    }

    #[inline]
    pub fn with(self, with: S::With) -> SendWithFut<'a, S, M> {
        SendWithFut { inner: self, with }
    }

    #[inline]
    pub fn recv(self) -> RequestFut<'a, S, M> {
        RequestFut(self)
    }

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

//-------------------------------------
// Request
//-------------------------------------

#[derive(Debug)]
pub struct RequestFut<'a, S: IsSender, M: Message>(SendFut<'a, S, M>);

impl<'a, S: IsSender, M: Message> RequestFut<'a, S, M> {
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

    #[inline]
    pub fn with(self, with: S::With) -> RequestWithFut<'a, S, M> {
        RequestWithFut(self.0.with(with))
    }

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

//-------------------------------------
// DynSendMsgWith
//-------------------------------------

#[derive(Debug)]
pub struct DynSendMsgWithFut<'a, S: IsSender, M>(SendMsgWithFut<'a, S, M>);

impl<'a, S: IsSender, M> DynSendMsgWithFut<'a, S, M> {
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

//-------------------------------------
// DynSendMsg
//-------------------------------------

#[derive(Debug)]
pub struct SendDynMsgFut<'a, S: IsSender, M>(SendMsgFut<'a, S, M>);

impl<'a, S: IsSender, M> SendDynMsgFut<'a, S, M> {
    #[inline]
    pub fn with(self, with: S::With) -> DynSendMsgWithFut<'a, S, M> {
        DynSendMsgWithFut(SendMsgWithFut {
            inner: self.0,
            with,
        })
    }

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

impl<'a, S: IsSender, M> IntoFuture for SendDynMsgFut<'a, S, M>
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

//-------------------------------------
// DynSendWith
//-------------------------------------

#[derive(Debug)]
pub struct DynSendWithFut<'a, S: IsSender, M: Message>(SendWithFut<'a, S, M>);

impl<'a, S: IsSender, M: Message> DynSendWithFut<'a, S, M> {
    #[inline]
    fn with_msg(self) -> (DynSendMsgWithFut<'a, S, M>, M::Output) {
        let (send_with_msg, output) = self.0.with_msg();
        (DynSendMsgWithFut(send_with_msg), output)
    }

    #[inline]
    pub fn recv(self) -> DynRequestWithFut<'a, S, M> {
        DynRequestWithFut(self)
    }

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

//-------------------------------------
// DynRequestWith
//-------------------------------------

#[derive(Debug)]
pub struct DynRequestWithFut<'a, S: IsSender, M: Message>(DynSendWithFut<'a, S, M>);

impl<'a, S: IsSender, M: Message> DynRequestWithFut<'a, S, M> {
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

//-------------------------------------
// DynSend
//-------------------------------------

#[derive(Debug)]
pub struct DynSendFut<'a, S: IsSender, M: Message>(SendFut<'a, S, M>);

impl<'a, S: IsSender, M: Message> DynSendFut<'a, S, M> {
    #[inline]
    pub fn with(self, with: S::With) -> DynSendWithFut<'a, S, M> {
        DynSendWithFut(SendWithFut {
            inner: self.0,
            with,
        })
    }

    #[inline]
    pub fn recv(self) -> DynRequestFut<'a, S, M> {
        DynRequestFut(self)
    }

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

//-------------------------------------
// DynRequest
//-------------------------------------

#[derive(Debug)]
pub struct DynRequestFut<'a, S: IsSender, M: Message>(DynSendFut<'a, S, M>);

impl<'a, S: IsSender, M: Message> DynRequestFut<'a, S, M> {
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
