use crate::AnyBox;
use std::{any::TypeId, fmt::Debug, marker::PhantomData};

// /// A protocol defines the messages it accepts by implementing this trait.
// ///
// /// Usually the protocol is an enum that implements [`ProtocolFor<M>`] for each variant.
// /// It should not be implemented for `Self`, but only for the variants.
// ///
// /// This can be derived on an enum using [`macro@Protocol`](crate)
// pub trait FromInto<M>: Sized {
//     /// Convert a message into the protocol.
//     #[must_use]
//     fn from_msg(msg: M) -> Self;

//     /// Attemppt to convert the protocol into the message (variant).
//     fn try_into_msg(self) -> Result<M, Self>;
// }

/// A variant of [`ProtocolFor`] that can be used for dynamic dispatch, meaning that
/// at runtime, [`Message`](crate)s are checked for acceptance.
///
/// This can be derived on an enum using [`macro@DynProtocol`]
pub trait BoxedFromInto: Sized {
    /// Get the list of accepted [`Message`]s.
    #[must_use]
    fn accepts_all() -> &'static [TypeId];

    /// Attempt to convert a bxed [`Message`] into the full protocol (enum),
    /// failing if the message is not accepted.
    fn try_from_boxed_msg<W: 'static>(msg: BoxedMsg<W>) -> Result<(Self, W), BoxedMsg<W>>;

    /// Convert the full protocol (enum) into a boxed [`Message`].
    #[must_use]
    fn into_boxed_msg<W: Send + 'static>(self, with: W) -> BoxedMsg<W>;
}

/// A marker trait for [`AcceptsDyn`], to signal that a message is accepted.
///
/// When implemented on a type that is not actually accepted, the `send`
/// methods will panic.
///
/// This can be derived on an enum using [`macro@AcceptsDyn`]
pub trait DynAcceptMarker<M, W = ()> {}

pub struct BoxedMsg<W = ()> {
    w: PhantomData<fn() -> W>,
    inner: AnyBox,
}

impl<W> Debug for BoxedMsg<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoxedMsgWith").field(&self.inner).finish()
    }
}

impl<W> BoxedMsg<W> {
    pub fn new<M>(t: M, with: W) -> Self
    where
        M: Send + 'static,
        W: Send + 'static,
    {
        Self {
            w: PhantomData,
            inner: Box::new((t, with)),
        }
    }

    pub fn downcast<M>(self) -> Result<(M, W), Self>
    where
        M: 'static,
        W: 'static,
    {
        match self.inner.downcast::<(M, W)>() {
            Ok(t) => Ok(*t),
            Err(boxed) => Err(Self {
                w: PhantomData,
                inner: boxed,
            }),
        }
    }
}
