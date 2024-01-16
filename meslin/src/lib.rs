#![allow(clippy::type_complexity)]
#![deny(unsafe_code)]
//! # Meslin
//! Meslin is a Rust library offering ergonomic wrappers for channels like [`mpmc`]
//! and [`broadcast`]. It's designed to ease the creation of actor systems by adding
//! user-friendly features, without being tied to any specific runtime. This makes
//! it compatible with various runtimes such as `tokio`, `smol`, or `async-std`.It intentionally
//! steers clear of incorporating supervisory functions or other complex features, focusing
//! instead on simplicity and non-interference.
//!
//! Meslin is designed with a zero-cost abstraction principle in mind, ensuring that
//! its ease of use and flexibility don't compromise performance. When not using any
//! dynamic features of the library, Meslin does not add any additional runtime
//! overhead compared to hand-written equivalents.
//!
//! ## Concepts
//! ### Messages
//! All messages that are sent through a channel must implement the [`Message`] trait.
//! The trait defines two associated types: [`Message::Input`] and [`Message::Output`].
//! When sending a message to an actor, you only need to provide the input type and if the message
//! is sent succesfully, the output type is returned.
//!
//! [`Message`] is implemented for a lot of common types, like `i32`, `String`, `Vec<T>`, etc.
//! Furthermore, it is implemented for [`Msg<M>`] and [`Request<A, B>`]. The first is a simple
//! wrapper that allows sending any type that does not implement [`trait@Message`]. The second is a
//! message that requires a response, i.e. the output is actually a [`oneshot::Receiver`].
//!
//! The [`macro@Message`] derive-macro can be used to derive `Message` for custom types.
//!
//! ### Protocols
//! Protocols define the messages that can be received by an actor. For every message `M` that
//! can be received, the protocol must implement [`From<M>`] and [`TryInto<M>`]. These traits can
//! be derived using the [`macro@From`] and [`macro@TryInto`] derive-macros.
//!
//! Optionally, the protocol can implement [`DynFromInto`] and [`trait@Accepts`] using the derive-macro [`macro@DynFromInto`].
//! This allows for conversion of senders into dynamic senders. See [`DynSender`] for more information.
//!
//! ### Senders
//! Senders are responsible for defining the delivery mechanism of a protocol. They implement
//! [`IsSender`] and can be used to send messages using [`Sends<M>`]. Examples of some default
//! senders are [`mpmc::Sender`], [`priority::Sender`] and the [`DynSender`].
//!
//! Most senders have their associated type [`IsSender::With`] set to `()`, meaning that they
//! don't require any additional data to send a message. However, some senders, like
//! [`priority::Sender`] do require additional data.
//! 
//! ### Send methods
//! The [`SendsExt`] and [`DynSendsExt`] traits provide a bunch of methods for sending messages.
//! The following are the modifier keywords and their meaning:
//! - `send`: The base method, that asynchronously sends a message and waits for space to become available.
//! - `request`: After sending the message, the [`Message::Output`] is awaited and returned immeadeately.
//! - `{...}_with`: Instead of using the default [`IsSender::With`] value, a custom value is given. 
//! - `try_{...}`:  Sends a message, returning an error if space is not available.
//! - `{...}_blocking`: Sends a message, blocking the current thread until space becomes available.
//! - `{...}_msg`: Instead of giving the [`Message::Input`], the message itself is given.
//! - `dyn_{...}`: Attempts to send a message, when it can not be statically verified that the actor will
//!   accept the message.
//!
//! ### Dynamic senders
//! A unique feature of Meslin is the transformation of senders into dynamic senders,
//! converting any sender into a [`dyn DynSends<W>`](DynSends). This allows for storage
//! of different sender types in the same data structure, like `Vec<T>`.
//!
//! [`DynSender`] provides an abstraction over a [`Box<dyn DynSends>`], allowing for
//! type-checked dynamic dispatch and conversions. For example,
//! if you have an [`mpmc::Sender<ProtocolA>`] and a [`broadcast::Sender<ProtocolB>`],
//! both accepting messages `Msg1` and `Msg2`, they can both be converted into
//! `DynSender<Accepts![Msg1, Msg2]>`. This dynamic sender then implements
//! `Sends<Msg1> + Sends<Msg2>`.
//!
//! The [`macro@Accepts`] macro can be used to define the accepted messages of a dynamic sender. Some
//! examples of dynamic sender conversions:
//! - `Accepts![Msg1, Msg2]` == `dyn AcceptsTwo<Msg1, Msg2>`.
//! - `DynSender<Accepts![Msg1, Msg2]>` can be converted into `DynSender<Accepts![Msg1]>`.
//! - `mpmc::Sender<ProtocolA>` can be converted into `DynSender<Accepts![Msg1, ...]>` as long as
//!   `ProtocolA` implements [`DynFromInto`] and `Accepts<Msg1> + Accepts<...> + ...`.
//! 
//! ## Cargo features
//! The following features are available:
//! - Default features: `["derive", "request", "mpmc", "broadcast", "priority"]`
//! - Additional features: `["watch"]""
//!
//! ## Basic example
//! ```
#![doc = include_str!("../examples/basic.rs")]
//! ```
//!
//! ## Advanced example
//! ```
#![doc = include_str!("../examples/advanced.rs")]
//! ```

mod dynamic;
mod errors;
mod features;
mod message;
mod sending;

pub use dynamic::*;
pub use errors::*;
pub use features::*;
pub use message::*;
pub use sending::*;

/// Re-export of [`type_sets`](::type_sets).
pub mod type_sets {
    pub use type_sets::*;
}
pub use type_sets::Set;

type AnyBox = Box<dyn std::any::Any + Send + 'static>;

trait ResultExt<T, E> {
    fn unwrap_silent(self) -> T;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn unwrap_silent(self) -> T {
        match self {
            Ok(t) => t,
            Err(_) => panic!("Unwrapping error {}", std::any::type_name::<Result<T, E>>()),
        }
    }
}
