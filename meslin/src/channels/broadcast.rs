use crate::*;
use std::fmt::Debug;

/// A wrapper around [`async_broadcast::Sender`].
pub struct Sender<P> {
    sender: async_broadcast::Sender<P>,
}

/// Re-export of [`async_broadcast::Receiver`].
pub use async_broadcast::Receiver;
use futures::Future;

impl<P> Sender<P> {
    pub fn inner(&self) -> &async_broadcast::Sender<P> {
        &self.sender
    }

    pub fn inner_mut(&mut self) -> &mut async_broadcast::Sender<P> {
        &mut self.sender
    }

    pub fn into_inner(self) -> async_broadcast::Sender<P> {
        self.sender
    }

    pub fn from_inner(sender: async_broadcast::Sender<P>) -> Self {
        Self { sender }
    }
}

impl<P> IsSender for Sender<P> {
    type With = ();

    fn is_closed(&self) -> bool {
        false
    }

    fn capacity(&self) -> Option<usize> {
        Some(self.sender.capacity())
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

impl<P: Clone + Send + Sync> IsStaticSender for Sender<P> {
    type Protocol = P;

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), SendNowError<(P, ())>> {
        this.sender
            .try_broadcast(protocol)
            .map(|_| ())
            .map_err(|e| match e {
                async_broadcast::TrySendError::Full(p) => SendNowError::Full((p, ())),
                async_broadcast::TrySendError::Closed(p) => SendNowError::Closed((p, ())),
                async_broadcast::TrySendError::Inactive(p) => SendNowError::Closed((p, ())),
            })
    }

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> impl Future<Output = Result<(), SendError<(P, ())>>> + Send {
        let fut = this.sender.broadcast_direct(protocol);
        async { fut.await.map(|_| ()).map_err(|e| SendError((e.0, ()))) }
    }
}

impl<P> Clone for Sender<P> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<P: Debug> std::fmt::Debug for Sender<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender")
            .field("sender", &self.sender)
            .finish()
    }
}

pub fn channel<P: Clone>(buffer: usize) -> (Sender<P>, async_broadcast::Receiver<P>) {
    let (sender, receiver) = async_broadcast::broadcast(buffer);
    (Sender { sender }, receiver)
}
