use crate::*;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct Request<A, B>(pub A, pub oneshot::Sender<B>);

impl<A, B> Request<A, B> {
    pub fn new(a: A) -> (Self, oneshot::Receiver<B>) {
        let (sender, receiver) = oneshot::channel();
        (Self(a, sender), receiver)
    }
}

impl<A, B> Message for Request<A, B>
where
    A: Send + 'static,
    B: Send + 'static,
{
    type Input = A;
    type Output = oneshot::Receiver<B>;

    fn create(input: Self::Input) -> (Self, Self::Output) {
        Self::new(input)
    }

    fn cancel(self, _: Self::Output) -> Self::Input {
        self.0
    }
}
