use tokio::sync::broadcast;
use crate::*;

pub struct Sender<P> {
    sender: broadcast::Sender<P>,
}

impl<P> Sender<P> {
    pub fn inner(&self) -> &broadcast::Sender<P> {
        &self.sender
    }

    pub fn into_inner(self) -> broadcast::Sender<P> {
        self.sender
    }

    pub fn from_inner(sender: broadcast::Sender<P>) -> Self {
        Self { sender }
    }
}

impl<P> SendProtocol for Sender<P>
where
    P: Send,
{
    type Protocol = P;

    async fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), Closed<Self::Protocol>> {
        match self.sender.send(protocol) {
            Ok(_amount) => Ok(()),
            Err(broadcast::error::SendError(protocol)) => Err(Closed(protocol)),
        }
    }

    fn send_protocol_blocking(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), Closed<Self::Protocol>> {
        match self.sender.send(protocol) {
            Ok(_amount) => Ok(()),
            Err(broadcast::error::SendError(protocol)) => Err(Closed(protocol)),
        }
    }
}

impl<P> TrySendProtocol for Sender<P> {
    type Protocol = P;

    fn try_send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), crate::sending::TrySendError<Self::Protocol>> {
        match self.sender.send(protocol) {
            Ok(_amount) => Ok(()),
            Err(broadcast::error::SendError(protocol)) => Err(TrySendError::Closed(protocol)),
        }
    }
}

impl<P> Clone for Sender<P> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

pub fn channel<P: Clone>(buffer: usize) -> (Sender<P>, broadcast::Receiver<P>) {
    let (sender, receiver) = broadcast::channel(buffer);
    (Sender { sender }, receiver)
}
