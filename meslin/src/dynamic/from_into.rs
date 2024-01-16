use crate::*;
use ::type_sets::Members;   

/// Trait that allows usage of dynamic senders for a protocol
///
/// This is usually derived on an enum using [`macro@DynFromInto`]
pub trait DynFromInto: Members + Sized {
    /// Attempt to convert a bxed [`Message`] into the full protocol (enum),
    /// failing if the message is not accepted.
    fn try_from_boxed_msg<W: 'static>(msg: BoxedMsg<W>) -> Result<(Self, W), BoxedMsg<W>>;

    /// Convert the full protocol (enum) into a boxed [`Message`].
    #[must_use]
    fn into_boxed_msg<W: Send + 'static>(self, with: W) -> BoxedMsg<W>;
}
