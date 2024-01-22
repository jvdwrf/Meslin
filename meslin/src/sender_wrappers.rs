use crate::*;
use core::future::Future;
use std::{fmt::Debug, marker::PhantomData};

/// A wrapper around a sender, which provides a default `with`-value.
#[derive(Debug)]
pub struct WithValueSender<T: IsSender> {
    inner: T,
    with: T::With,
}

impl<T> Clone for WithValueSender<T>
where
    T: Clone + IsSender,
    T::With: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            with: self.with.clone(),
        }
    }
}

impl<T: IsSender> WithValueSender<T> {
    pub fn new(sender: T, with: T::With) -> Self {
        Self {
            inner: sender,
            with,
        }
    }

    pub fn into_inner(self) -> (T, T::With) {
        (self.inner, self.with)
    }

    pub fn inner_ref(&self) -> (&T, &T::With) {
        (&self.inner, &self.with)
    }

    pub fn inner_mut(&mut self) -> (&mut T, &mut T::With) {
        (&mut self.inner, &mut self.with)
    }
}

impl<T> IsSender for WithValueSender<T>
where
    T: IsSender,
{
    type With = ();

    fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    fn capacity(&self) -> Option<usize> {
        self.inner.capacity()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn receiver_count(&self) -> usize {
        self.inner.receiver_count()
    }

    fn sender_count(&self) -> usize {
        self.inner.sender_count()
    }
}

impl<T> IsStaticSender for WithValueSender<T>
where
    T: IsStaticSender,
    T::With: Clone,
{
    type Protocol = T::Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: (),
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send {
        let fut = T::send_protocol_with(&this.inner, protocol, this.with.clone());
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
        match T::try_send_protocol_with(&this.inner, protocol, this.with.clone()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, _)| (protocol, with))),
        }
    }

    fn send_protocol_blocking_with(
        this: &Self,
        protocol: Self::Protocol,
        with: Self::With,
    ) -> Result<(), SendError<(Self::Protocol, Self::With)>> {
        match T::send_protocol_blocking_with(&this.inner, protocol, this.with.clone()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, _)| (protocol, with))),
        }
    }
}

/// A wrapper around a sender, which provides a mapping between the `with`-value of the sender and
/// a custom `with`-value.
pub struct MappedWithSender<
    T: IsSender,
    W,
    F1 = fn(W) -> <T as IsSender>::With,
    F2 = fn(<T as IsSender>::With) -> W,
> {
    sender: T,
    f1: F1,
    f2: F2,
    _marker: PhantomData<fn() -> W>,
}

impl<T: IsSender + Debug, W, F1, F2> Debug for MappedWithSender<T, W, F1, F2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MappedWithSender")
            .field("sender", &self.sender)
            .field("f1", &"...")
            .field("f2", &"...")
            .field("with", &std::any::type_name::<W>())
            .finish()
    }
}

impl<T: IsSender + Clone, W, F1: Clone, F2: Clone> Clone for MappedWithSender<T, W, F1, F2> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            f1: self.f1.clone(),
            f2: self.f2.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: IsSender, W, F1, F2> MappedWithSender<T, W, F1, F2> {
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

    pub fn inner_mut(&mut self) -> (&mut T, &F1, &mut F2) {
        (&mut self.sender, &mut self.f1, &mut self.f2)
    }
}

impl<T: IsSender, W, F1, F2> IsSender for MappedWithSender<T, W, F1, F2> {
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

impl<T, W, F1, F2> IsStaticSender for MappedWithSender<T, W, F1, F2>
where
    T: IsStaticSender,
    F1: Fn(W) -> T::With + Send + Sync,
    F2: Fn(T::With) -> W + Send + Sync,
{
    type Protocol = T::Protocol;

    fn send_protocol_with(
        this: &Self,
        protocol: Self::Protocol,
        with: W,
    ) -> impl Future<Output = Result<(), SendError<(Self::Protocol, Self::With)>>> + Send {
        let fut = T::send_protocol_with(&this.sender, protocol, (this.f1)(with));
        let f2 = &this.f2;
        async {
            match fut.await {
                Ok(()) => Ok(()),
                Err(e) => Err(e.map(|(protocol, with)| (protocol, f2(with)))),
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

    fn send_protocol_blocking_with(
        this: &Self,
        protocol: Self::Protocol,
        with: W,
    ) -> Result<(), SendError<(Self::Protocol, Self::With)>> {
        match T::send_protocol_blocking_with(&this.sender, protocol, (this.f1)(with)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.map(|(protocol, with)| (protocol, (this.f2)(with)))),
        }
    }
}
