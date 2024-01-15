use crate::*;
use core::future::Future;
use std::marker::PhantomData;

pub(super) struct DefaultWithWrapper<T>(T);

impl<T> IsSender for DefaultWithWrapper<T>
where
    T: IsSender,
{
    type With = ();

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

impl<T> SendsProtocol for DefaultWithWrapper<T>
where
    T: SendsProtocol,
    T::With: Default,
{
    type Protocol = T::Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send {
        let fut = T::send_protocol_with(&this.0, protocol, Default::default());
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(protocol, _)| (protocol, ()))),
            }
        }
    }

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        _with: (),
    ) -> Result<(), TrySendError<(Self::Protocol, Self::With)>> {
        match T::try_send_protocol_with(&this.0, protocol, Default::default()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, _)| (protocol, ()))),
        }
    }
}

pub(super) struct IntoWithWrapper<T, W>(T, PhantomData<fn() -> W>);

impl<T, W> IsSender for IntoWithWrapper<T, W>
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

impl<T, W> SendsProtocol for IntoWithWrapper<T, W>
where
    T: SendsProtocol,
    W: Into<T::With>,
    T::With: Into<W>,
{
    type Protocol = T::Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send {
        let fut = T::send_protocol_with(&this.0, protocol, with.into());
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(protocol, with)| (protocol, with.into()))),
            }
        }
    }

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> Result<(), TrySendError<(Self::Protocol, Self::With)>> {
        match T::try_send_protocol_with(&this.0, protocol, with.into()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, with)| (protocol, with.into()))),
        }
    }
}
