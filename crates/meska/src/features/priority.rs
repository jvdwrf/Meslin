use crate::*;
use async_priority_channel as prio;
use std::fmt::Debug;

pub trait PriorityProtocol<M> {}




pub struct Sender<P, O: Ord> {
    sender: async_priority_channel::Sender<P, O>,
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

impl<P: Send, O: Ord + Send> SendProtocol for Sender<P, O> {
    type Protocol = P;
    type Error = prio::SendError<()>;

    async fn send_protocol(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol, Self::Error>> {
        todo!()
        // self.sender
        //     .send(protocol, todo!())
        //     .await
        //     .map_err(|e| SendError::new(e.0, prio::SendError(())))
    }
}

impl<P, O: Ord> SendProtocolNow for Sender<P, O> {
    type Protocol = P;
    type Error = prio::TrySendError<()>;

    fn send_protocol_now(
        &self,
        protocol: Self::Protocol,
    ) -> Result<(), SendError<Self::Protocol, Self::Error>> {
        todo!()
        // self.sender.try_send(protocol).map_err(|e| match e {
        //     prio::TrySendError::Full(protocol) => {
        //         SendError::new(protocol, prio::TrySendError::Full(()))
        //     }
        //     prio::TrySendError::Closed(protocol) => {
        //         SendError::new(protocol, prio::TrySendError::Closed(()))
        //     }
        // })
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
