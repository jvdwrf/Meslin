use crate::*;
use type_sets::{Members, SubsetOf};

/// Trait implemented for all senders that can transform into a [`DynSender<T, W>`].
pub trait IntoDynSender<T, W = ()> {
    /// Transform the sender into a [`DynSender<T, W>`].
    fn into_dyn_sender(self) -> DynSender<T, W>;
}

impl<T, W, R> IntoDynSender<R, W> for DynSender<T, W>
where
    R: SubsetOf<T>,
{
    fn into_dyn_sender(self) -> DynSender<R, W> {
        self.transform()
    }
}

impl<T, W, R> IntoDynSender<R, W> for T
where
    T: IsStaticSender<With = W> + IsDynSender<With = W>,
    R: SubsetOf<T::Protocol>,
{
    fn into_dyn_sender(self) -> DynSender<R, W> {
        DynSender::new(self)
    }
}

impl<W> IntoDynSender<Set![], W> for Box<dyn IsDynSender<With = W>> {
    fn into_dyn_sender(self) -> DynSender![; W] {
        DynSender::from_inner_unchecked(self)
    }
}

/// Trait implemented for all senders that can dynamically try-transform into a [`DynSender<T, W>`].
pub trait TryIntoDynSender<T, W = ()>: Sized {
    /// Attempt to transform the sender into a [`DynSender<T, W>`], failing if the protocol does not
    /// accept the messages.
    fn try_into_dyn_sender(self) -> Result<DynSender<T, W>, Self>;
}

impl<T, W, R> TryIntoDynSender<R, W> for DynSender<T, W>
where
    R: Members,
    T: 'static,
    W: 'static,
{
    fn try_into_dyn_sender(self) -> Result<DynSender<R, W>, Self> {
        self.try_transform()
    }
}

impl<T, W, R> TryIntoDynSender<R, W> for T
where
    T: IsStaticSender<With = W> + IsDynSender<With = W>,
    R: SubsetOf<T::Protocol>,
{
    fn try_into_dyn_sender(self) -> Result<DynSender<R, W>, Self> {
        Ok(DynSender::new(self))
    }
}

impl<W, R> TryIntoDynSender<R, W> for Box<dyn IsDynSender<With = W>>
where
    W: 'static,
    R: Members + 'static,
{
    fn try_into_dyn_sender(self) -> Result<DynSender<R, W>, Self> {
        DynSender::try_from_inner(self)
    }
}
