use std::any::TypeId;

use crate::*;

pub trait Accepts<M, W = ()> {}

pub trait TryAccept {
    fn accepts_all() -> &'static [TypeId];
}

impl<M, W, T> Accepts<M, W> for T where T: From<M> + TryInto<M> {}

pub trait TransformFrom<T: ?Sized> {}

pub mod marker {
    use std::{any::TypeId, sync::OnceLock};

    use crate::*;

    macro_rules! create_markers {
        ($(
            $n:literal $accepts:ident<$($gen:ident),*> $(:)? $($prev_accept:path),*;
        )*) => {
            $(
                /// A marker trait to indicate that a process accepts a message.
                pub trait $accepts<$($gen),*>: $($prev_accept +)* {}

                // Make the marker-traits auto-traits.
                impl<$($gen,)* S: ?Sized> $accepts<$($gen),*> for S where S: $($prev_accept +)* {}

                // And implement the correct FromSpec implementations
                impl<$($gen,)* S: ?Sized> TransformFrom<S> for crate::Accepts!($($gen,)*)
                where
                    S: $accepts<$($gen),*>
                {}

                impl<$($gen: 'static,)*> TryAccept for crate::Accepts!($($gen,)*)
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
    );
}

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
    }

#[cfg(test)]
mod test {
    use super::*;
    #[allow(clippy::type_complexity, unused)]
    fn compilation_test<T1, T2, T3, T4, T5, T6, T7, T8>() {
        let _: Accepts!();
        let _: Accepts!(T1);
        let _: Accepts!(T1, T2);
        let _: Accepts!(T1, T2, T3);
        let _: Accepts!(T1, T2, T3, T4);
        let _: Accepts!(T1, T2, T3, T4, T5);
        let _: Accepts!(T1, T2, T3, T4, T5, T6);
        let _: Accepts!(T1, T2, T3, T4, T5, T6, T7);
        let _: Accepts!(T1, T2, T3, T4, T5, T6, T7, T8);
    }

    #[test]
    fn type_ids_same() {
        assert_eq!(<Accepts!(u32)>::accepts_all()[0], TypeId::of::<u32>(),);
        assert_eq!(<Accepts!(u32, u64)>::accepts_all()[0], TypeId::of::<u32>(),);
        assert_eq!(<Accepts!(u32, u64)>::accepts_all()[1], TypeId::of::<u64>(),);
    }
}
