use crate::*;
use core::future::Future;
use std::marker::PhantomData;

/// A wrapper around a sender, that always sends a default `with` value.
///
/// The mapping is from `W` to `T::With`.
pub struct WithValueSender<T: IsSender, W = ()> {
    sender: T,
    with: T::With,
    _marker: PhantomData<fn() -> W>,
}

impl<T, W> Clone for WithValueSender<T, W>
where
    T: Clone + IsSender,
    T::With: Clone,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            with: self.with.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: IsSender, W> WithValueSender<T, W> {
    pub fn new(sender: T, with: T::With) -> Self {
        Self {
            sender,
            with,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(self) -> (T, T::With) {
        (self.sender, self.with)
    }

    pub fn inner_ref(&self) -> (&T, &T::With) {
        (&self.sender, &self.with)
    }

    pub fn inner_mut(&mut self) -> (&mut T, &mut T::With) {
        (&mut self.sender, &mut self.with)
    }
}

impl<T, W> IsSender for WithValueSender<T, W>
where
    T: IsSender,
{
    type With = W;

    fn is_closed(&self) -> bool {
        self.sender.is_closed()
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

impl<T, W> SendsProtocol for WithValueSender<T, W>
where
    T: SendsProtocol,
    T::With: Clone,
    W: Send,
{
    type Protocol = T::Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: W,
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send {
        let fut = T::send_protocol_with(&this.sender, protocol, this.with.clone());
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
        match T::try_send_protocol_with(&this.sender, protocol, this.with.clone()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, _)| (protocol, with))),
        }
    }
}

pub struct MappedWithSender<T, F1, F2, W> {
    sender: T,
    f1: F1,
    f2: F2,
    _marker: PhantomData<fn() -> W>,
}

impl<T, F1, F2, W> Clone for MappedWithSender<T, F1, F2, W>
where
    T: Clone + IsSender,
    F1: Clone,
    F2: Clone,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            f1: self.f1.clone(),
            f2: self.f2.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T, F1, F2, W> MappedWithSender<T, F1, F2, W>
where
    T: IsSender,
    F1: FnMut(W) -> T::With,
    F2: FnMut(T::With) -> W,
{
    pub fn new(sender: T, f1: F1, f2: F2) -> Self {
        Self {
            sender,
            f1,
            f2,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(self) -> (T, F1, F2) {
        (self.sender, self.f1, self.f2)
    }

    pub fn inner_ref(&self) -> (&T, &F1, &F2) {
        (&self.sender, &self.f1, &self.f2)
    }

    pub fn inner_mut(&mut self) -> (&mut T, &mut F1, &mut F2) {
        (&mut self.sender, &mut self.f1, &mut self.f2)
    }
}

impl<T, F1, F2, W> IsSender for MappedWithSender<T, F1, F2, W>
where
    T: IsSender,
    F1: FnMut(W) -> T::With,
    F2: FnMut(T::With) -> W,
{
    type With = W;

    fn is_closed(&self) -> bool {
        self.sender.is_closed()
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

impl<T, F1, F2, W> SendsProtocol for MappedWithSender<T, F1, F2, W>
where
    T: SendsProtocol + Send + Sync,
    F1: Fn(W) -> T::With + Send + Sync,
    F2: Fn(T::With) -> W + Send + Sync,
    W: Send,
{
    type Protocol = T::Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: W,
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send {
        let fut = T::send_protocol_with(&this.sender, protocol, (this.f1)(with));
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(protocol, with)| (protocol, (this.f2)(with)))),
            }
        }
    }

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), TrySendError<(Self::Protocol, Self::With)>> {
        match T::try_send_protocol_with(&this.sender, protocol, (this.f1)(with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, with)| (protocol, (this.f2)(with)))),
        }
    }
}
