use crate::{
    address::Address,
    child::Child,
    inbox::Inbox,
    message::Message,
    spawn::spawn,
    specification::{AddressSpec, ChannelSpec, ChildSpec, InboxSpec, SendError, TrySendError},
};
use futures::{Future, FutureExt, Stream};
use std::{
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

/// A basic tokio task
#[derive(Debug)]
pub struct Task<O> {
    handle: tokio::task::JoinHandle<O>,
}

impl<O: Send + 'static + Debug> ChildSpec for Task<O> {
    type Config = ();
    type Output = O;
    type OutputError = tokio::task::JoinError;

    fn poll_child_exit(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<O, Self::OutputError>> {
        self.handle.poll_unpin(cx)
    }

    fn spawn_future<F>(_cfg: Option<Self::Config>, f: F) -> Self
    where
        F: Future<Output = Self::Output> + Send + 'static,
        F::Output: Send + 'static,
    {
        Self {
            handle: tokio::spawn(f),
        }
    }
}

/// The simplest kind of inbox
#[derive(Debug)]
pub struct Sender<T> {
    sender: tokio::sync::mpsc::Sender<T>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<T: Send + 'static> AddressSpec for Sender<T> {
    type Protocol = T;
    type Output = ();

    fn is_alive(&self) -> bool {
        todo!()
    }

    fn poll_address(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }

    async fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol>> {
        self.sender.send(protocol).await.map_err(|e| SendError(e.0))
    }

    fn try_send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), TrySendError<Self::Protocol>> {
        self.sender.try_send(protocol).map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Closed(e) => TrySendError::Closed(e),
            tokio::sync::mpsc::error::TrySendError::Full(e) => TrySendError::Full(e),
        })
    }
}

pub struct Receiver<T> {
    receiver: tokio::sync::mpsc::Receiver<T>,
}

impl<T: Send + 'static> InboxSpec for Receiver<T> {
    type Receives = T;
}

impl<T: Send + 'static> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

struct Mpsc<T>(PhantomData<T>);

impl<T: Send + 'static> ChannelSpec for Mpsc<T> {
    type Config = usize;
    type InboxSpec = Receiver<T>;
    type AddressSpec = Sender<T>;

    fn create(cfg: Option<Self::Config>) -> (Self::InboxSpec, Self::AddressSpec) {
        let (sender, receiver) = tokio::sync::mpsc::channel(cfg.unwrap_or(100));
        (Receiver { receiver }, Sender { sender })
    }
}

#[derive(Debug)]
pub struct Request<A, B>(pub A, pub tokio::sync::oneshot::Sender<B>);

impl<A, B> Message for Request<A, B> {
    type Input = A;
    type Output = tokio::sync::oneshot::Receiver<B>;

    fn create(from: Self::Input) -> (Self, Self::Output) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        (Self(from, sender), receiver)
    }

    fn cancel(self, _: Self::Output) -> Self::Input {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::{Mpsc, Request, Task};
    use crate::{
        address::Address,
        spawn::{spawn, spawn_default},
        Spec,
    };

    #[derive(Debug /* Message2 */)]
    pub struct SayHello(pub String);

    #[derive(Debug /* Protocol, Message, DynProtocol */)]
    pub enum MyProtocol {
        String(SayHello),
        Request(Request<SayHello, u8>),
        Req2(Request<usize, u16>),
    }

    async fn spawn_test() {
        let (child, address) = spawn::<Task<_>, Mpsc<u32>, _, _>(
            None,
            Some(10),
            |inbox, address| async move { "hello" },
        );

        let (child, address) = spawn_default::<Task<_>, Mpsc<MyProtocol>, _, _>(
            |inbox, address| async move { "hello" },
        );

        address.send::<SayHello>("hello").await.unwrap();
        let _response = address
            .send::<Request<SayHello, u8>>("hello")
            .await
            .unwrap()
            .await
            .unwrap();

        let _response = address
            .request::<Request<SayHello, u8>>("hello")
            .await
            .unwrap();

        let address = address
            .into_spec::<Spec![SayHello, Request<SayHello, u8>]>();
            // .into_spec::<Spec![SayHello]>();

        address.send_dyn::<SayHello>("hi").await.unwrap();
        address
            .send_dyn::<Request<SayHello, u8>>("hi")
            .await
            .unwrap()
            .await
            .unwrap();
        address
            .request_dyn::<Request<SayHello, u8>>("hi")
            .await
            .unwrap();
    }

    mod msg_ex {
        use crate::{
            message::Message,
            message::{Protocol, DynamicProtocol, ProtocolMarker},
            tokio_task::Request,
            AnyBox,
        };
        use std::{
            any::{Any, TypeId},
            sync::OnceLock,
        };

        use super::{MyProtocol, SayHello};

        impl<T> From<T> for SayHello
        where
            T: Into<String>,
        {
            fn from(s: T) -> Self {
                Self(s.into())
            }
        }

        impl Message for SayHello {
            type Input = Self;
            type Output = ();

            fn create(from: Self::Input) -> (Self, Self::Output) {
                (from, ())
            }

            fn cancel(self, _: Self::Output) -> Self::Input {
                self
            }
        }

        impl ProtocolMarker<SayHello> for MyProtocol {}
        impl ProtocolMarker<Request<SayHello, u8>> for MyProtocol {}
        impl ProtocolMarker<Request<usize, u16>> for MyProtocol {}

        impl DynamicProtocol for MyProtocol {
            fn accepted() -> &'static [TypeId] {
                static ACCEPTS: OnceLock<[TypeId; 3]> = OnceLock::new();
                ACCEPTS.get_or_init(|| {
                    [
                        TypeId::of::<SayHello>(),
                        TypeId::of::<Request<SayHello, u8>>(),
                        TypeId::of::<Request<usize, u16>>(),
                    ]
                })
            }

            fn try_from_boxed_msg(msg: AnyBox) -> Result<Self, AnyBox> {
                let msg_id = (*msg).type_id();

                if msg_id == TypeId::of::<SayHello>() {
                    Ok(MyProtocol::String(*msg.downcast::<SayHello>().unwrap()))
                } else if msg_id == TypeId::of::<Request<SayHello, u8>>() {
                    Ok(MyProtocol::Request(
                        *msg.downcast::<Request<SayHello, u8>>().unwrap(),
                    ))
                } else if msg_id == TypeId::of::<Request<usize, u16>>() {
                    Ok(MyProtocol::Req2(
                        *msg.downcast::<Request<usize, u16>>().unwrap(),
                    ))
                } else {
                    Err(msg)
                }
            }

            fn into_boxed_msg(self) -> AnyBox {
                match self {
                    Self::String(msg) => Box::new(msg),
                    Self::Request(msg) => Box::new(msg),
                    Self::Req2(msg) => Box::new(msg),
                }
            }
        }

        impl Message for MyProtocol {
            type Input = Self;
            type Output = ();

            fn create(from: Self::Input) -> (Self, Self::Output) {
                (from, ())
            }

            fn cancel(self, _: Self::Output) -> Self::Input {
                self
            }
        }

        impl Protocol<SayHello> for MyProtocol {
            fn from_msg(msg: SayHello) -> Self {
                Self::String(msg)
            }

            fn try_into_msg(self) -> Result<SayHello, Self> {
                match self {
                    Self::String(s) => Ok(s),
                    _ => Err(self),
                }
            }
        }

        impl Protocol<Request<SayHello, u8>> for MyProtocol {
            fn from_msg(msg: Request<SayHello, u8>) -> Self {
                Self::Request(msg)
            }

            fn try_into_msg(self) -> Result<Request<SayHello, u8>, Self> {
                match self {
                    Self::Request(r) => Ok(r),
                    _ => Err(self),
                }
            }
        }

        impl Protocol<Request<usize, u16>> for MyProtocol {
            fn from_msg(msg: Request<usize, u16>) -> Self {
                Self::Req2(msg)
            }

            fn try_into_msg(self) -> Result<Request<usize, u16>, Self> {
                match self {
                    Self::Req2(r) => Ok(r),
                    _ => Err(self),
                }
            }
        }
    }
}
