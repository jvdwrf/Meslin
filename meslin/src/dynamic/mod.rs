use crate::*;
use std::{fmt::Debug, marker::PhantomData};

mod sends_ext;
pub use sends_ext::*;
mod wrappers;
pub use wrappers::*;
mod sends;
pub use sends::*;
mod from_into;
pub use from_into::*;
mod sender;
pub use sender::*;

/// A boxed message with a `with` value, used for dynamic dispatch.
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
    pub fn new<M>(msg: M, with: W) -> Self
    where
        M: Send + 'static,
        W: Send + 'static,
    {
        Self {
            w: PhantomData,
            inner: Box::new((msg, with)),
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
