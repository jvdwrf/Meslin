use crate::AnyBox;
use std::any::TypeId;

/// A protocol defines the messages it accepts by implementing this trait.
///
/// Usually the protocol is an enum that implements [`ProtocolFor<M>`] for each variant.
/// It should not be implemented for `Self`, but only for the variants.
///
/// This can be derived on an enum using [`macro@Protocol`]
pub trait ProtocolFor<M>: Sized {
    /// Convert a message into the protocol.
    fn from_msg(msg: M) -> Self;

    /// Attemppt to convert the protocol into the message (variant).
    fn try_into_msg(self) -> Result<M, Self>;
}

/// A variant of [`ProtocolFor`] that can be used for dynamic dispatch, meaning that
/// at runtime, [`Message`](crate)s are checked for acceptance.
///
/// This can be derived on an enum using [`macro@DynProtocol`]
pub trait DynProtocol: Sized {
    // todo: fix bug, where a message can be converted from a boxed message
    // to the protocol, and then into a boxed message with another type-id.
    // We can use the marker-type for this instead of the real Accepts<M>.

    /// Check whether the given [`Message`] is accepted.
    fn accepts(type_id: TypeId) -> bool {
        Self::accepted().contains(&type_id)
    }
    /// Get the list of accepted [`Message`]s.
    fn accepted() -> &'static [TypeId];

    /// Attempt to convert a bxed [`Message`] into the full protocol (enum),
    /// failing if the message is not accepted.
    fn try_from_boxed_msg(msg: AnyBox) -> Result<Self, AnyBox>;

    /// Convert the full protocol (enum) into a boxed [`Message`].
    fn into_boxed_msg(self) -> AnyBox;
}

/// A marker trait for [`AcceptsDyn`], to signal that a message is accepted.
///
/// When implemented on a type that is not actually accepted, the `send`
/// methods will panic.
///
/// This can be derived on an enum using [`macro@AcceptsDyn`]
pub trait DynProtocolMarker<M> {}
