use crate::*;

/// Wrapper around [`flume::Sender`].
pub struct Sender<P> {
    sender: flume::Sender<P>,
}

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

impl<P: Send> SendProtocol for Sender<P> {
    type Protocol = P;
    type SendError = flume::SendError<()>;
    type SendNowError = flume::TrySendError<()>;

    async fn send_protocol_with(
        &self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), Error<(Self::Protocol, ()), Self::SendError>> {
        self.sender
            .send_async(protocol)
            .await
            .map_err(|e| Error::new((e.0, ()), flume::SendError(())))
    }

    fn send_protocol_blocking_with(
        &self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), Error<(Self::Protocol, ()), Self::SendError>> {
        self.sender
            .send(protocol)
            .map_err(|e| Error::new((e.0, ()), flume::SendError(())))
    }

    fn send_protocol_now_with(
        &self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), Error<(Self::Protocol, ()), Self::SendNowError>> {
        self.sender.try_send(protocol).map_err(|e| match e {
            flume::TrySendError::Disconnected(protocol) => {
                Error::new((protocol, ()), flume::TrySendError::Disconnected(()))
            }
            flume::TrySendError::Full(protocol) => {
                Error::new((protocol, ()), flume::TrySendError::Full(()))
            }
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
