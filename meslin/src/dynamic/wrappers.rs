use crate::*;
use core::future::Future;
use std::marker::PhantomData;

/// A wrapper around a sender, which provides a default `with`-value.
pub struct WithValueSender<T: IsSender> {
    sender: T,
    with: T::With,
}

impl<T> Clone for WithValueSender<T>
where
    T: Clone + IsSender,
    T::With: Clone,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            with: self.with.clone(),
        }
    }
}

impl<T: IsSender> WithValueSender<T> {
    pub fn new(sender: T, with: T::With) -> Self {
        Self { sender, with }
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

impl<T> IsSender for WithValueSender<T>
where
    T: IsSender,
{
    type With = ();

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

impl<T> SendsProtocol for WithValueSender<T>
where
    T: SendsProtocol,
    T::With: Clone,
{
    type Protocol = T::Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: (),
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send {
        let fut = T::send_protocol_with(&this.sender, protocol, this.with.clone());
        async move {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(protocol, _)| (protocol, with))),
            }
        }
    }

    fn try_send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: (),
    ) -> Result<(), TrySendError<(Self::Protocol, Self::With)>> {
        match T::try_send_protocol_with(&this.sender, protocol, this.with.clone()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, _)| (protocol, with))),
        }
    }
}

/// A wrapper around a sender, which provides a mapping between the `with`-value of the sender and
/// a custom `with`-value.
pub struct MappedWithSender<T: IsSender, W> {
    sender: T,
    f1: fn(W) -> T::With,
    f2: fn(T::With) -> W,
    _marker: PhantomData<fn() -> W>,
}

impl<T: IsSender + Clone, W> Clone for MappedWithSender<T, W> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            f1: self.f1,
            f2: self.f2,
            _marker: PhantomData,
        }
    }
}

impl<T: IsSender, W> MappedWithSender<T, W> {
    pub fn new(sender: T, f1: fn(W) -> T::With, f2: fn(T::With) -> W) -> Self {
        Self {
            sender,
            f1,
            f2,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(self) -> (T, fn(W) -> T::With, fn(T::With) -> W) {
        (self.sender, self.f1, self.f2)
    }

    pub fn inner_ref(&self) -> (&T, &fn(W) -> T::With, &fn(T::With) -> W) {
        (&self.sender, &self.f1, &self.f2)
    }

    pub fn inner_mut(&mut self) -> (&mut T, &mut fn(W) -> T::With, &mut fn(T::With) -> W) {
        (&mut self.sender, &mut self.f1, &mut self.f2)
    }
}

impl<T: IsSender, W> IsSender for MappedWithSender<T, W> {
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

impl<T, W> SendsProtocol for MappedWithSender<T, W>
where
    T: SendsProtocol + Send + Sync,
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
