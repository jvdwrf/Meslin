use crate::*;
use ::type_sets::Members;
use std::{fmt::Debug, marker::PhantomData};

/// Trait that allows usage of dynamic senders for a protocol
///
/// This is usually derived on an enum using [`macro@FromIntoBoxed`]
pub trait FromIntoBoxed: Members + Sized {
    /// Attempt to convert a bxed [`Message`] into the full protocol (enum),
    /// failing if the message is not accepted.
    fn try_from_boxed_msg<W: 'static>(msg: BoxedMsg<W>) -> Result<(Self, W), BoxedMsg<W>>;

    /// Convert the full protocol (enum) into a boxed [`Message`].
    #[must_use]
    fn into_boxed_msg<W: Send + 'static>(self, with: W) -> BoxedMsg<W>;
}

/// A boxed message with a `with` value, used for dynamic dispatch.
pub struct BoxedMsg<W = ()> {
    msg: AnyBox,
    _with: PhantomData<fn() -> W>,
}

impl<W> Debug for BoxedMsg<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoxedMsgWith").field(&self.msg).finish()
    }
}

impl<W> BoxedMsg<W> {
    pub fn new<M>(msg: M, with: W) -> Self
    where
        M: Send + 'static,
        W: Send + 'static,
    {
        Self {
            _with: PhantomData,
            msg: Box::new((msg, with)),
        }
    }

    pub fn downcast<M>(self) -> Result<(M, W), Self>
    where
        M: 'static,
        W: 'static,
    {
        match self.msg.downcast::<(M, W)>() {
            Ok(t) => Ok(*t),
            Err(boxed) => Err(Self {
                _with: PhantomData,
                msg: boxed,
            }),
        }
    }
}
