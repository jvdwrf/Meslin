use std::{fmt::Debug, sync::Arc};

use crate::*;
use tokio::sync::watch;

/// Wrapper around [`tokio::sync::watch::Sender`].
pub struct Sender<P> {
    sender: Arc<watch::Sender<P>>,
}

/// Re-export of [`tokio::sync::watch::Receiver`].
pub use watch::Receiver;

impl<P> Sender<P> {
    pub fn inner(&self) -> &Arc<watch::Sender<P>> {
        &self.sender
    }

    pub fn inner_mut(&mut self) -> &mut Arc<watch::Sender<P>> {
        &mut self.sender
    }

    pub fn into_inner(self) -> Arc<watch::Sender<P>> {
        self.sender
    }

    pub fn from_inner(sender: Arc<watch::Sender<P>>) -> Self {
        Self { sender }
    }
}

impl<P> IsSender for Sender<P> {
    type With = ();

    fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    fn capacity(&self) -> Option<usize> {
        None
    }

    fn len(&self) -> usize {
        1
    }

    fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    fn sender_count(&self) -> usize {
        1
    }
}

impl<P: Clone + Send + Sync> IsStaticSender for Sender<P> {
    type Protocol = P;

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), TrySendError<(P, ())>> {
        this.sender
            .send(protocol)
            .map_err(|e| TrySendError::Closed((e.0, ())))
    }

    async fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), SendError<(Self::Protocol, ())>> {
        this.sender.send(protocol).map_err(|e| SendError((e.0, ())))
    }
}

impl<P: Debug> Debug for Sender<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender")
            .field("sender", &self.sender)
            .finish()
    }
}

impl<P> Clone for Sender<P> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

pub fn channel<P>(init: P) -> (Sender<P>, watch::Receiver<P>) {
    let (sender, receiver) = watch::channel::<P>(init);
    (
        Sender {
            sender: Arc::new(sender),
        },
        receiver,
    )
}
