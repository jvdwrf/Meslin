use crate::*;
use std::any::TypeId;

/// Trait that allows usage of dynamic senders for a protocol
///
/// This is usually derived on an enum using [`macro@DynFromInto`]
pub trait DynFromInto: AcceptsAll + Sized {
    /// Attempt to convert a bxed [`Message`] into the full protocol (enum),
    /// failing if the message is not accepted.
    fn try_from_boxed_msg<W: 'static>(msg: BoxedMsg<W>) -> Result<(Self, W), BoxedMsg<W>>;

    /// Convert the full protocol (enum) into a boxed [`Message`].
    #[must_use]
    fn into_boxed_msg<W: Send + 'static>(self, with: W) -> BoxedMsg<W>;
}

/// Marker trait that defines which messages are dynamically accepted by a protocol.
///
/// This is usually derived on an enum using [`macro@DynFromInto`]
pub trait Accepts<M> {}

/// Trait that specifies a list of messages accepted by a protocol.
///
/// This is usually derived on an enum using [`macro@DynFromInto`]
pub trait AcceptsAll {
    fn accepts_all() -> &'static [TypeId];
}

/// Marker trait that indicates a subset of T is accepted.
pub trait AcceptsSubsetOf<T: ?Sized> {}

/// Macro that allows for dynamic specification of accepted messages.
/// 
/// It expands as follows:
/// - `Accepts![]` == `dyn AcceptsNone`
/// - `Accepts![T1]` == `dyn AcceptsOne<T1>`
/// - `Accepts![T1, T2]` == `dyn AcceptsTwo<T1, T2>`
/// - etc.
///
/// Some usage examples:
/// - `DynSender<Accepts![u32, u64]>`
/// - `sender.into_dyn::<Accepts![u32, u64]>()`
/// - `dyn_sender.transform::<Accepts![u32, u64]>()`
#[macro_export]
macro_rules! Accepts {
    ($(,)?) => {
        dyn $crate::marker::AcceptsNone
    };
    ($t1:ty $(,)?) => {
        dyn $crate::marker::AcceptsOne<$t1>
    };
    ($t1:ty, $t2:ty $(,)?) => {
        dyn $crate::marker::AcceptsTwo<$t1, $t2>
    };
    ($t1:ty, $t2:ty, $t3:ty $(,)?) => {
        dyn $crate::marker::AcceptsThree<$t1, $t2, $t3>
    };
    ($t1:ty, $t2:ty, $t3:ty, $t4:ty $(,)?) => {
        dyn $crate::marker::AcceptsFour<$t1, $t2, $t3, $t4>
    };
    ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty $(,)?) => {
        dyn $crate::marker::AcceptsFive<$t1, $t2, $t3, $t4, $t5>
    };
    ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty, $t6:ty $(,)?) => {
        dyn $crate::marker::AcceptsSix<$t1, $t2, $t3, $t4, $t5, $t6>
    };
    ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty, $t6:ty, $t7:ty $(,)?) => {
        dyn $crate::marker::AcceptsSeven<$t1, $t2, $t3, $t4, $t5, $t6, $t7>
    };
    ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty, $t6:ty, $t7:ty, $t8:ty $(,)?) => {
        dyn $crate::marker::AcceptsEight<$t1, $t2, $t3, $t4, $t5, $t6, $t7, $t8>
    };
    ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty, $t6:ty, $t7:ty, $t8:ty, $t9:ty $(,)?) => {
        dyn $crate::marker::AcceptsNine<$t1, $t2, $t3, $t4, $t5, $t6, $t7, $t8, $t9>
    };
    ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty, $t6:ty, $t7:ty, $t8:ty, $t9:ty, $t10:ty $(,)?) => {
        dyn $crate::marker::AcceptsTen<$t1, $t2, $t3, $t4, $t5, $t6, $t7, $t8, $t9, $t10>
    };
}

pub mod marker {
    //! Marker traits for dynamic protocols
    use crate::*;
    use std::{any::TypeId, sync::OnceLock};

    macro_rules! create_markers {
        ($(
            $n:literal $accepts:ident<$($gen:ident),*> $(:)? $($prev_accept:path),*;
        )*) => {
            $(
                // Create the marker-traits
                /// Marker trait indicating which messages are accepted by a protocol.
                /// 
                /// Use the [`macro@Accepts`] macro instead of this.
                pub trait $accepts<$($gen),*>: $($prev_accept +)* {}

                // Make the marker-traits auto-traits.
                impl<$($gen,)* S: ?Sized> $accepts<$($gen),*> for S where S: $($prev_accept +)* {}

                // And implement the correct FromSpec implementations
                impl<$($gen,)* S: ?Sized> AcceptsSubsetOf<S> for crate::Accepts!($($gen,)*)
                where
                    S: $accepts<$($gen),*>
                {}

                impl<$($gen: 'static,)*> AcceptsAll for crate::Accepts!($($gen,)*)
                {
                    fn accepts_all() -> &'static [std::any::TypeId] {
                        static LOCK: OnceLock<[TypeId; $n]> = OnceLock::new();
                        LOCK.get_or_init(|| [ $(TypeId::of::<$gen>()),* ])
                    }
                }

                impl<$($gen: 'static,)*> std::fmt::Debug for crate::Accepts!($($gen,)*) {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.debug_tuple(stringify!($accepts))
                            $(.field(&stringify!($gen)))*
                            .finish()
                    }
                }
            )*
        };
    }

    create_markers!(
        0 AcceptsNone<>;
        1 AcceptsOne<T1>: AcceptsNone, Accepts<T1>;
        2 AcceptsTwo<T1, T2>: AcceptsOne<T1>, Accepts<T2>;
        3 AcceptsThree<T1, T2, T3>: AcceptsTwo<T1, T2>, Accepts<T3>;
        4 AcceptsFour<T1, T2, T3, T4>: AcceptsThree<T1, T2, T3>, Accepts<T4>;
        5 AcceptsFive<T1, T2, T3, T4, T5>: AcceptsFour<T1, T2, T3, T4>, Accepts<T5>;
        6 AcceptsSix<T1, T2, T3, T4, T5, T6>: AcceptsFive<T1, T2, T3, T4, T5>, Accepts<T6>;
        7 AcceptsSeven<T1, T2, T3, T4, T5, T6, T7>: AcceptsSix<T1, T2, T3, T4, T5, T6>, Accepts<T7>;
        8 AcceptsEight<T1, T2, T3, T4, T5, T6, T7, T8>: AcceptsSeven<T1, T2, T3, T4, T5, T6, T7>, Accepts<T8>;
        9 AcceptsNine<T1, T2, T3, T4, T5, T6, T7, T8, T9>: AcceptsEight<T1, T2, T3, T4, T5, T6, T7, T8>, Accepts<T9>;
        10 AcceptsTen<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>: AcceptsNine<T1, T2, T3, T4, T5, T6, T7, T8, T9>, Accepts<T10>;
    );
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(clippy::type_complexity, unused)]
    fn compilation_test<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>() {
        let _: Accepts!();
        let _: Accepts!(T1);
        let _: Accepts!(T1, T2);
        let _: Accepts!(T1, T2, T3);
        let _: Accepts!(T1, T2, T3, T4);
        let _: Accepts!(T1, T2, T3, T4, T5);
        let _: Accepts!(T1, T2, T3, T4, T5, T6);
        let _: Accepts!(T1, T2, T3, T4, T5, T6, T7);
        let _: Accepts!(T1, T2, T3, T4, T5, T6, T7, T8);
        let _: Accepts!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        let _: Accepts!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
    }

    #[test]
    fn type_ids_same() {
        assert_eq!(<Accepts!(u32)>::accepts_all()[0], TypeId::of::<u32>(),);
        assert_eq!(<Accepts!(u32, u64)>::accepts_all()[0], TypeId::of::<u32>(),);
        assert_eq!(<Accepts!(u32, u64)>::accepts_all()[1], TypeId::of::<u64>(),);
    }
}
