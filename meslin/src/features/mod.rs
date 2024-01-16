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

/// A oneshot channel using [`oneshot`](::oneshot).
#[cfg(feature = "request")]
pub mod oneshot;
#[cfg(feature = "request")]
pub use oneshot::Request;

#[cfg(feature = "derive")]
mod derive {
    #[allow(unused_imports)]
    use crate::*;

    /// Re-export of [`derive_more::From`](derive_more::derive::From).
    pub use derive_more::derive::From;
    /// Re-export of [`derive_more::TryInto`](derive_more::derive::TryInto).
    pub use derive_more::derive::TryInto;

    /// Macro to derive [`Message`] for a type.
    /// 
    /// The message's input is `Self` and the output is `()`. For more complicated messages,
    /// implement [`Message`] manually.
    /// 
    /// It can be useful to derive [`macro@From`] as well, optionally with  the 
    /// `#[from(forward)]` attribute.
    pub use meslin_derive::Message;

    /// Macro to derive [`trait@DynFromInto`] and [`AsSet`](type_sets::AsSet)
    /// for an enum.
    /// 
    /// This derive macro implements all necessary traits to use the protocol with dynamic senders.
    /// Usually, this would be combined with the derive macros [`macro@From`] and [`macro@TryInto`].
    pub use meslin_derive::DynFromInto;
}
pub use derive::*;

pub(crate) type BoxedSender<W = ()> = Box<dyn crate::DynSends<With = W>>;
