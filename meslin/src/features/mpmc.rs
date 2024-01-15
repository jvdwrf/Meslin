use crate::*;

/// Wrapper around [`flume::Sender`].
pub struct Sender<P> {
    sender: flume::Sender<P>,
}

pub use flume::Receiver;

impl<P> Sender<P> {
    pub fn inner(&self) -> &flume::Sender<P> {
        &self.sender
    }

    pub fn into_inner(self) -> flume::Sender<P> {
        self.sender
    }

    pub fn inner_mut(&mut self) -> &mut flume::Sender<P> {
        &mut self.sender
    }

    pub fn from_inner(sender: flume::Sender<P>) -> Self {
        Self { sender }
    }
}

impl<P> IsSender for Sender<P> {
    type With = ();

    fn is_closed(&self) -> bool {
        self.sender.is_disconnected()
    }

    fn capacity(&self) -> Option<usize> {
        self.sender.capacity()
    }

    fn len(&self) -> usize {
        self.sender.len()
    }

    fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    fn sender_count(&self) -> usize {
        self.sender.sender_count()
    }
}

impl<P: Send> SendsProtocol for Sender<P> {
    type Protocol = P;

    async fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), SendError<(Self::Protocol, ())>> {
        this.sender
            .send_async(protocol)
            .await
            .map_err(|e| SendError((e.0, ())))
    }

    fn send_protocol_blocking_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), SendError<(Self::Protocol, ())>> {
        this.sender.send(protocol).map_err(|e| SendError((e.0, ())))
    }

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), TrySendError<(Self::Protocol, ())>> {
        this.sender.try_send(protocol).map_err(|e| match e {
            flume::TrySendError::Disconnected(protocol) => TrySendError::Closed((protocol, ())),
            flume::TrySendError::Full(protocol) => TrySendError::Full((protocol, ())),
        })
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

pub fn bounded<P>(cap: usize) -> (Sender<P>, flume::Receiver<P>) {
    let (sender, receiver) = flume::bounded(cap);
    (Sender { sender }, receiver)
}

pub fn unbounded<P>() -> (Sender<P>, flume::Receiver<P>) {
    let (sender, receiver) = flume::unbounded();
    (Sender { sender }, receiver)
}
