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

impl<P> IsSender for Sender<P> {
    fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    fn capacity(&self) -> Option<usize> {
        None
    }

    fn len(&self) -> usize {
        1
    }

    fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    fn sender_count(&self) -> usize {
        1
    }
}

impl<P: Clone + Send + Sync> SendProtocol for Sender<P> {
    type Protocol = P;

    fn try_send_protocol_with(
        &self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), TrySendError<(P, ())>> {
        self.sender
            .send(protocol)
            .map_err(|e| TrySendError::Closed((e.0, ())))
    }

    async fn send_protocol_with(
        &self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), SendError<(Self::Protocol, ())>> {
        self.sender.send(protocol).map_err(|e| SendError((e.0, ())))
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
