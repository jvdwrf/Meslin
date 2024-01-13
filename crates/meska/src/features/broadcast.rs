use crate::*;
use tokio::sync::broadcast;

pub struct Sender<P> {
    sender: broadcast::Sender<P>,
}

impl<P> Sender<P> {
    pub fn inner(&self) -> &broadcast::Sender<P> {
        &self.sender
    }

    pub fn inner_mut(&mut self) -> &mut broadcast::Sender<P> {
        &mut self.sender
    }

    pub fn into_inner(self) -> broadcast::Sender<P> {
        self.sender
    }

    pub fn from_inner(sender: broadcast::Sender<P>) -> Self {
        Self { sender }
    }
}

impl<P> SendProtocolNow for Sender<P> {
    type Protocol = P;
    type Error = broadcast::error::SendError<()>;

    fn send_protocol_now(&self, protocol: Self::Protocol) -> Result<(), SendError<P, Self::Error>> {
        match self.sender.send(protocol) {
            Ok(_amount) => Ok(()),
            Err(broadcast::error::SendError(protocol)) => {
                Err(SendError::new(protocol, broadcast::error::SendError(())))
            }
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

impl<P> std::fmt::Debug for Sender<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender")
            .field("sender", &self.sender)
            .finish()
    }
}

pub fn channel<P: Clone>(buffer: usize) -> (Sender<P>, broadcast::Receiver<P>) {
    let (sender, receiver) = broadcast::channel(buffer);
    (Sender { sender }, receiver)
}
