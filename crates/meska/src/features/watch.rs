use std::fmt::Debug;

use crate::*;
use tokio::sync::watch;

pub struct Sender<P> {
    sender: watch::Sender<P>,
}

impl<P> Sender<P> {
    pub fn inner(&self) -> &watch::Sender<P> {
        &self.sender
    }

    pub fn inner_mut(&mut self) -> &mut watch::Sender<P> {
        &mut self.sender
    }

    pub fn into_inner(self) -> watch::Sender<P> {
        self.sender
    }

    pub fn from_inner(sender: watch::Sender<P>) -> Self {
        Self { sender }
    }
}

impl<P> SendExt for Sender<P> {}

impl<P: Clone> SendProtocolNow for Sender<P> {
    type Protocol = P;
    type Error = watch::error::SendError<()>;

    fn send_protocol_now_with(
        &self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), Error<(P, ()), Self::Error>> {
        self.sender
            .send(protocol)
            .map_err(|e| Error::new((e.0, ()), watch::error::SendError(())))
    }
}

impl<P: Debug> Debug for Sender<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender")
            .field("sender", &self.sender)
            .finish()
    }
}

pub fn channel<P>(init: P) -> (Sender<P>, watch::Receiver<P>) {
    let (sender, receiver) = watch::channel::<P>(init);
    (Sender { sender }, receiver)
}
