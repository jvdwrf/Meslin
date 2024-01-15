use crate::*;
use tokio::sync::broadcast;

pub struct Sender<P> {
    sender: broadcast::Sender<P>,
}

pub use broadcast::Receiver;

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
    type With = ();

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
        // https://docs.rs/async-broadcast/latest/async_broadcast/
        todo!("Switch to another library that implements sender_count for broadcast")
    }
}

impl<P: Send> SendsProtocol for Sender<P> {
    type Protocol = P;

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), TrySendError<(P, ())>> {
        this.sender
            .send(protocol)
            .map(|_| ())
            .map_err(|e| TrySendError::Closed((e.0, ())))
    }

    async fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), SendError<(Self::Protocol, ())>> {
        this.sender
            .send(protocol)
            .map(|_| ())
            .map_err(|e| SendError((e.0, ())))
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
