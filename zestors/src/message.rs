use crate::AnyBox;
use std::any::TypeId;
use std::{num::*, rc::Rc, sync::Arc};

/// All messages must implement this trait.
///
/// It defines two types: `Input` and `Output`.
/// - [`Message::Input`] is the type that is converted into the message.
/// - [`Message::Output`] is the type that is returned when the message is sent.
pub trait Message: Sized {
    /// The type that is converted into the message.
    type Input;

    /// The type that is returned when the message is sent.
    type Output;

    /// Create a message from the given input.
    fn create(from: Self::Input) -> (Self, Self::Output);

    /// Cancel the message and return the input.
    fn cancel(self, returned: Self::Output) -> Self::Input;
}

/// A protocol defines the messages it accepts by implementing this trait.
///
/// Usually the protocol is an enum that implements [`Accepts`] for each variant.
/// These variants can then be sent using an [`Address`].
///
/// This can be derived on an enum using [`macro@Accepts`]
pub trait Protocol<M>: Sized {
    /// Convert a message into the full protocol (enum).
    fn from_msg(msg: M) -> Self;

    /// Convert the full protocol (enum) into a message.
    fn try_into_msg(self) -> Result<M, Self>;
}

/// A variant of [`Accepts`] that can be used for dynamic dispatch.
/// At runtime, it is checked whether the [`Message`] is accepted.
///
/// This can be derived on an enum using [`macro@AcceptsDyn`]
pub trait DynamicProtocol: Sized {
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
pub trait ProtocolMarker<M> {}

/// A simple wrapper for any type that does not implement [`Message`].
/// This is useful for sending types that are not owned by the sender.
///
/// [`Msg<T>`] implements [`Message`] for any type `T`.
#[derive(Debug, Clone, Copy)]
pub struct Msg<T>(pub T);

impl<T> Message for Msg<T> {
    type Input = T;
    type Output = ();

    fn create(from: Self::Input) -> (Self, Self::Output) {
        (Self(from), ())
    }

    fn cancel(self, _: Self::Output) -> Self::Input {
        self.0
    }
}

macro_rules! common_messages {
    (0;
        $($ty:ty),* $(,)?
    ) => { $(common_messages!(impl<> $ty);)* };
    (1;
        $($ty:ty),* $(,)?
    ) => { $(common_messages!(impl<T1> $ty);)* };
    (2;
        $($ty:ty),* $(,)?
    ) => { $(common_messages!(impl<T1, T2> $ty);)* };
    (
        impl<$($gen:ident),*> $ty:ty
    ) => {
        impl<$($gen),*> Message for $ty {
            type Input = $ty;
            type Output = ();

            fn create(from: Self::Input) -> (Self, Self::Output) {
                (from, ())
            }

            fn cancel(self, _: Self::Output) -> Self::Input {
                self
            }
        }
    }
}

common_messages!(0;
    char, String, bool, &'static str,
    usize, u8, u16, u32, u64,
    isize, i8, i16, i32, i64,
    f32, f64,
    NonZeroUsize, NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64,
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64,
);
common_messages!(1;
    Option<T1>,
    Vec<T1>,
    Box<T1>,
    Arc<T1>,
    Rc<T1>,
    &'static [T1],
);
common_messages!(2;
    Result<T1, T2>,
);

macro_rules! tuple_messages {
    ($(
        ($($t:ident),* $(,)?)
    ),* $(,)?) => {
        $(
            impl<$($t,)*> Message for ($($t,)*) {
                type Input = Self;
                type Output = ();

                fn create(from: Self::Input) -> (Self, Self::Output) {
                    (from, ())
                }

                fn cancel(self, _: Self::Output) -> Self::Input {
                    self
                }
            }
        )*
    };
}

tuple_messages!(
    (),
    (T1,),
    (T1, T2),
    (T1, T2, T3),
    (T1, T2, T3, T4),
    (T1, T2, T3, T4, T5),
    (T1, T2, T3, T4, T5, T6),
    (T1, T2, T3, T4, T5, T6, T7),
    (T1, T2, T3, T4, T5, T6, T7, T8),
    (T1, T2, T3, T4, T5, T6, T7, T8, T9),
    (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
);
