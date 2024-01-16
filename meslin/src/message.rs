use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    num::*,
    rc::Rc,
    sync::Arc,
};

/// Trait that defines how a message is created and canceled.
///
/// It defines two types: `Input` and `Output`:
/// - [`Message::Input`] is the type that is converted into the message.
/// - [`Message::Output`] is the type that is returned when the message is sent.
///
/// Whenever a message is sent, it is converted into a message and an output.
/// Then, upon a succesful send, the output is returned. If sending failed, the
/// message is canceled and the input is returned again.
///
/// Message is automatically implemented for a lot of common types, like `i32`,
/// `String`, `Vec<T>`, etc.
pub trait Message: Sized {
    /// The type that is converted into the message.
    type Input;

    /// The type that is returned when the message is sent.
    type Output: Send;

    /// Create a message from the given input.
    ///
    /// Called when the message is sent.
    fn create(input: Self::Input) -> (Self, Self::Output);

    /// Cancel the message and return the input.
    ///
    /// Called when sending the message failed.
    fn cancel(self, output: Self::Output) -> Self::Input;
}

/// A simple wrapper for any type that does not implement [`Message`].
/// 
/// This is useful for sending types that are not owned by the sender, since 
/// [`Msg<T>`] implements [`Message`] for any type `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msg<T>(pub T);

impl<T: Send + 'static> Message for Msg<T> {
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
        impl<$($gen: Send + Sync + 'static),*> Message for $ty {
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
    Vec<T1>, HashSet<T1>, BTreeSet<T1>, LinkedList<T1>, BinaryHeap<T1>, VecDeque<T1>,
    Box<T1>,
    Arc<T1>,
    Rc<T1>,
    &'static [T1],
);
common_messages!(2;
    Result<T1, T2>,
    HashMap<T1, T2>, BTreeMap<T1, T2>,

);

macro_rules! tuple_messages {
    ($(
        ($($t:ident),* $(,)?)
    ),* $(,)?) => {
        $(
            impl<$($t: Send + 'static,)*> Message for ($($t,)*) {
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
