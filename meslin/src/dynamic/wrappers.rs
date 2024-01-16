use crate::*;
use core::future::Future;
use std::marker::PhantomData;

/// Maps a `IsSender::With: Default` to `IsSender<With = W>`.
pub(super) struct MappedWithSender<T, W>(T, PhantomData<fn() -> W>);

impl<T: Clone, W> Clone for MappedWithSender<T, W> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T, W> MappedWithSender<T, W> {
    pub(super) fn new(t: T) -> Self {
        Self(t, PhantomData)
    }

    pub(super) fn into_inner(self) -> T {
        self.0
    }
}

impl<T, W> IsSender for MappedWithSender<T, W>
where
    T: IsSender,
{
    type With = W;

    fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    fn capacity(&self) -> Option<usize> {
        self.0.capacity()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn receiver_count(&self) -> usize {
        self.0.receiver_count()
    }

    fn sender_count(&self) -> usize {
        self.0.sender_count()
    }
}

impl<T, W> SendsProtocol for MappedWithSender<T, W>
where
    T: SendsProtocol,
    T::With: Default,
    W: Send,
{
    type Protocol = T::Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: W,
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send {
        let fut = T::send_protocol_with(&this.0, protocol, Default::default());
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(protocol, _)| (protocol, with))),
            }
        }
    }

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), TrySendError<(Self::Protocol, Self::With)>> {
        match T::try_send_protocol_with(&this.0, protocol, Default::default()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, _)| (protocol, with))),
        }
    }
}
