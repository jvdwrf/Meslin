use crate::*;
use async_priority_channel as prio;
use std::fmt::Debug;

pub struct Sender<P, O: Ord> {
    sender: prio::Sender<P, O>,
}

impl<P, O: Ord> Sender<P, O> {
    pub fn inner(&self) -> &prio::Sender<P, O> {
        &self.sender
    }

    pub fn into_inner(self) -> prio::Sender<P, O> {
        self.sender
    }

    pub fn inner_mut(&mut self) -> &mut prio::Sender<P, O> {
        &mut self.sender
    }

    pub fn from_inner(sender: prio::Sender<P, O>) -> Self {
        Self { sender }
    }
}

impl<P, O: Ord> SendExt for Sender<P, O> {}

impl<P: Send, O: Ord + Send> SendProtocol<O> for Sender<P, O> {
    type Protocol = P;
    type Error = prio::SendError<()>;

    async fn send_protocol_with(
        &self,
        protocol: Self::Protocol,
        with: O,
    ) -> Result<(), Error<(Self::Protocol, O), Self::Error>> {
        self.sender
            .send(protocol, with)
            .await
            .map_err(|e| Error::new(e.0, prio::SendError(())))
    }
}

impl<P, O: Ord> SendProtocolNow<O> for Sender<P, O> {
    type Protocol = P;
    type Error = prio::TrySendError<()>;

    fn send_protocol_now_with(
        &self,
        protocol: Self::Protocol,
        with: O,
    ) -> Result<(), Error<(Self::Protocol, O), Self::Error>> {
        self.sender.try_send(protocol, with).map_err(|e| match e {
            prio::TrySendError::Full(protocol) => {
                Error::new(protocol, prio::TrySendError::Full(()))
            }
            prio::TrySendError::Closed(protocol) => {
                Error::new(protocol, prio::TrySendError::Closed(()))
            }
        })
    }
}

impl<P: Debug, O: Ord + Debug> Debug for Sender<P, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender")
            .field("sender", &self.sender)
            .finish()
    }
}

pub fn channel<P, O: Ord>(cap: Option<u64>) -> (Sender<P, O>, prio::Receiver<P, O>) {
    let (sender, receiver) = match cap {
        Some(size) => prio::bounded(size),
        None => prio::unbounded(),
    };
    (Sender { sender }, receiver)
}
