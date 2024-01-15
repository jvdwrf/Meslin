#[cfg(feature = "request")]
mod request;
#[cfg(feature = "request")]
pub use request::*;

use crate::DynSends;

#[cfg(feature = "derive")]
pub use {
    derive_more::{From, TryInto},
    meslin_derive::*,
};
// #[cfg(feature = "derive")]

#[cfg(feature = "broadcast")]
pub mod broadcast;

#[cfg(feature = "watch")]
pub mod watch;

#[cfg(feature = "mpmc")]
pub mod mpmc;

#[cfg(feature = "priority")]
pub mod priority;

pub(crate) type BoxedSender<W = ()> = Box<dyn DynSends<With = W>>;
