use crate::*;

/// A [`Message`] with input `A`, returning a response `B`.
///
/// This implements [`Message`] with [`oneshot::Receiver`] as output.
#[derive(Debug)]
pub struct Request<A, B> {
    pub msg: A,
    pub tx: ::oneshot::Sender<B>,
}

/// Re-export of [`oneshot::Receiver`](::oneshot::Receiver).
pub use ::oneshot::Receiver;
/// Re-export of [`oneshot::Sender`](::oneshot::Sender).
pub use ::oneshot::Sender;

impl<A, B> Request<A, B> {
    pub fn new(msg: A) -> (Self, ::oneshot::Receiver<B>) {
        let (sender, receiver) = ::oneshot::channel();
        (Self { msg, tx: sender }, receiver)
    }
}

impl<A, B> Message for Request<A, B>
where
    A: Send + 'static,
    B: Send + 'static,
{
    type Input = A;
    type Output = ::oneshot::Receiver<B>;

    fn create(input: Self::Input) -> (Self, Self::Output) {
        Self::new(input)
    }

    fn cancel(self, _: Self::Output) -> Self::Input {
        self.msg
    }
}
