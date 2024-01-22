use std::fmt::Debug;

use crate::*;
use type_sets::{Members, SubsetOf};

pub trait IntoSender<T> {
    /// Transform the sender into a [`DynSender<T, W>`].
    fn into_sender(self) -> T;
}

impl<T, W, R> IntoSender<DynSender<R, W>> for DynSender<T, W>
where
    R: SubsetOf<T>,
{
    fn into_sender(self) -> DynSender<R, W> {
        self.transform()
    }
}

impl<T, W, R> IntoSender<DynSender<R, W>> for T
where
    T: IsStaticSender<With = W> + IsDynSender<With = W>,
    R: SubsetOf<T::Protocol>,
{
    fn into_sender(self) -> DynSender<R, W> {
        DynSender::new(self)
    }
}

impl<W> IntoSender<DynSender<Set![], W>> for Box<dyn IsDynSender<With = W>> {
    fn into_sender(self) -> DynSender![; W] {
        DynSender::from_inner_unchecked(self)
    }
}

impl<T> IntoSender<T> for T
where
    T: IsStaticSender,
{
    fn into_sender(self) -> T {
        self
    }
}

impl<T> IntoSender<Box<dyn IsDynSender<With = T::With>>> for T
where
    T: IsStaticSender + Clone + 'static + Send + Sync + Debug,
    T::Protocol: DynProtocol,
    T::With: Send,
{
    fn into_sender(self) -> Box<dyn IsDynSender<With = T::With>> {
        Box::new(self)
    }
}

/// Trait implemented for all senders that can dynamically try-transform into a [`DynSender<T, W>`].
pub trait TryIntoSender<T>: Sized {
    /// Attempt to transform the sender into a [`DynSender<T, W>`], failing if the protocol does not
    /// accept the messages.
    fn try_into_sender(self) -> Result<T, Self>;
}

impl<T, W, R> TryIntoSender<DynSender<R, W>> for DynSender<T, W>
where
    R: Members,
    T: 'static,
    W: 'static,
{
    fn try_into_sender(self) -> Result<DynSender<R, W>, Self> {
        self.try_transform()
    }
}

impl<W, R> TryIntoSender<DynSender<R, W>> for Box<dyn IsDynSender<With = W>>
where
    W: 'static,
    R: Members + 'static,
{
    fn try_into_sender(self) -> Result<DynSender<R, W>, Self> {
        DynSender::try_from_inner(self)
    }
}

impl<T, R> TryIntoSender<R> for T
where
    T: IntoSender<R> + IsStaticSender,
{
    fn try_into_sender(self) -> Result<R, Self> {
        Ok(self.into_sender())
    }
}
