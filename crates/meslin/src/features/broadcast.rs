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

impl<P> IsSender for Sender<P> {
    fn is_closed(&self) -> bool {
        false
    }

    fn capacity(&self) -> Option<usize> {
        None
    }

    fn len(&self) -> usize {
        self.sender.len()
    }

    fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    fn sender_count(&self) -> usize {
        1
    }
}

impl<P: Send> SendProtocol for Sender<P> {
    type Protocol = P;
    type SendNowError = broadcast::error::SendError<()>;
    type SendError = Self::SendNowError;

    fn send_protocol_now_with(
        &self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), Error<(P, ()), Self::SendNowError>> {
        match self.sender.send(protocol) {
            Ok(_amount) => Ok(()),
            Err(broadcast::error::SendError(protocol)) => {
                Err(Error::new((protocol, ()), broadcast::error::SendError(())))
            }
        }
    }

    async fn send_protocol_with(
        &self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), Error<(Self::Protocol, ()), Self::SendError>> {
        self.send_protocol_now_with(protocol, ())
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
