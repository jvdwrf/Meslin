/// A broadcast channel using [`async_broadcast`].
#[cfg(feature = "broadcast")]
pub mod broadcast;

/// A watch channel using [`tokio::sync::watch`].
#[cfg(feature = "watch")]
pub mod watch;

/// An mpmc channel using [`flume`].
#[cfg(feature = "mpmc")]
pub mod mpmc;

/// A priority channel using [`async_priority_channel`].
#[cfg(feature = "priority")]
pub mod priority;

#[cfg(feature = "request")]
mod request;
#[cfg(feature = "request")]
pub use request::*;

#[cfg(feature = "derive")]
pub use {
    derive_more::{From, TryInto},
    meslin_derive::*,
};

pub(crate) type BoxedSender<W = ()> = Box<dyn crate::DynSends<With = W>>;
