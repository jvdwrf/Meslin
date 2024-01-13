use crate::*;
use tokio::sync::mpsc;

pub struct Sender<P> {
    sender: mpsc::Sender<P>,
}

impl<P> Sender<P> {
    pub fn inner(&self) -> &mpsc::Sender<P> {
        &self.sender
    }

    pub fn into_inner(self) -> mpsc::Sender<P> {
        self.sender
    }

    pub fn inner_mut(&mut self) -> &mut mpsc::Sender<P> {
        &mut self.sender
    }

    pub fn from_inner(sender: mpsc::Sender<P>) -> Self {
        Self { sender }
    }
}

impl<P: Send> SendProtocol for Sender<P> {
    type Protocol = P;
    type Error = mpsc::error::SendError<()>;

    async fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol, Self::Error>> {
        self.sender
            .send(protocol)
            .await
            .map_err(|e| SendError::new(e.0, mpsc::error::SendError(())))
    }
}

impl<P> SendProtocolNow for Sender<P> {
    type Protocol = P;
    type Error = mpsc::error::TrySendError<()>;

    fn send_protocol_now(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol, Self::Error>> {
        self.sender.try_send(protocol).map_err(|e| match e {
            mpsc::error::TrySendError::Closed(protocol) => {
                SendError::new(protocol, mpsc::error::TrySendError::Closed(()))
            }
            mpsc::error::TrySendError::Full(protocol) => {
                SendError::new(protocol, mpsc::error::TrySendError::Full(()))
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

pub fn channel<P>(buffer: usize) -> (Sender<P>, mpsc::Receiver<P>) {
    let (sender, receiver) = mpsc::channel(buffer);
    (Sender { sender }, receiver)
}

pub struct UnboundedSender<P> {
    sender: mpsc::UnboundedSender<P>,
}

impl<P> UnboundedSender<P> {
    pub fn inner(&self) -> &mpsc::UnboundedSender<P> {
        &self.sender
    }

    pub fn into_inner(self) -> mpsc::UnboundedSender<P> {
        self.sender
    }

    pub fn from_inner(sender: mpsc::UnboundedSender<P>) -> Self {
        Self { sender }
    }
}

impl<P> SendProtocol for UnboundedSender<P>
where
    P: Send + 'static,
{
    type Protocol = P;
    type Error = ();

    async fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol, Self::Error>> {
        self.sender
            .send(protocol)
            .map_err(|e| SendError::new(e.0, ()))
    }
}

impl<P> SendProtocolNow for UnboundedSender<P>
where
    P: Send + 'static,
{
    type Protocol = P;
    type Error = ();

    fn send_protocol_now(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol, Self::Error>> {
        self.sender
            .send(protocol)
            .map_err(|e| SendError::new(e.0, ()))
    }
}

impl<P> Clone for UnboundedSender<P> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<P> std::fmt::Debug for UnboundedSender<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnboundedSender")
            .field("sender", &self.sender)
            .finish()
    }
}

pub fn unbounded_channel<P>() -> (UnboundedSender<P>, mpsc::UnboundedReceiver<P>) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (UnboundedSender { sender }, receiver)
}
