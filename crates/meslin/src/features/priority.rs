use crate::*;
use async_priority_channel as prio;
use std::{f32::consts::E, fmt::Debug};

/// Wrapper around [`async_priority_channel::Sender`].
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

impl<P, O: Ord> IsSender for Sender<P, O> {
    fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    fn capacity(&self) -> Option<usize> {
        self.sender.capacity().map(|c| c.try_into().unwrap())
    }

    fn len(&self) -> usize {
        self.sender.len().try_into().unwrap()
    }

    fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    fn sender_count(&self) -> usize {
        self.sender.sender_count()
    }
}

impl<P: Send, O: Ord + Send> SendProtocol<O> for Sender<P, O> {
    type Protocol = P;

    async fn send_protocol_with(
        &self,
        protocol: Self::Protocol,
        with: O,
    ) -> Result<(), SendError<(Self::Protocol, O)>> {
        self.sender
            .send(protocol, with)
            .await
            .map_err(|e| SendError(e.0))
    }

    fn try_send_protocol_with(
        &self,
        protocol: Self::Protocol,
        with: O,
    ) -> Result<(), TrySendError<(Self::Protocol, O)>> {
        self.sender.try_send(protocol, with).map_err(|e| match e {
            prio::TrySendError::Full(e) => TrySendError::Full(e),
            prio::TrySendError::Closed(e) => TrySendError::Closed(e),
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

pub fn bounded<P, O: Ord>(size: usize) -> (Sender<P, O>, prio::Receiver<P, O>) {
    let (sender, receiver) = prio::bounded(size.try_into().unwrap());
    (Sender { sender }, receiver)
}

pub fn unbounded<P, O: Ord>() -> (Sender<P, O>, prio::Receiver<P, O>) {
    let (sender, receiver) = prio::unbounded();
    (Sender { sender }, receiver)
}
