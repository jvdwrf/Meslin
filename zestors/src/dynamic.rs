use crate::{
    message::{DynamicProtocol, Message, Protocol, ProtocolMarker},
    specification::{
        AddressSpec, DynAddressSpec, FromSpec, SendDynError, SendError, SendNowError,
        TrySendDynError,
    },
    AnyBox, ResultExt,
};
use futures::{future::BoxFuture, Future};
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    pin,
    task::{Context, Poll},
};

use self::marker::AcceptsNone;

//-------------------------------------
// Dyn
//-------------------------------------

pub struct DynSpec<T: ?Sized = dyn AcceptsNone> {
    spec: Box<dyn AcceptDynObjectSafe>,
    t: PhantomData<fn() -> T>,
}

impl<T: ?Sized> DynSpec<T> {
    pub fn downcast<S: 'static>(self) -> Result<S, Self> {
        match self.downcast_ref::<S>() {
            Some(_) => Ok(*self.spec.into_any().downcast::<S>().unwrap()),
            None => Err(self),
        }
    }

    pub fn downcast_ref<S: 'static>(&self) -> Option<&S> {
        self.spec.as_any().downcast_ref::<S>()
    }
}

impl<T: ?Sized> AddressSpec for DynSpec<T> {
    type Protocol = DynProtocol<DynSpec<T>>;
    type Output = ();
    fn is_alive(&self) -> bool {
        todo!()
    }

    fn poll_address(self: pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }

    #[allow(clippy::manual_async_fn)]
    fn send_protocol(
        &self,
        _: Self::Protocol,
    ) -> impl Future<Output = Result<(), SendError<Self::Protocol>>> + Send + '_ {
        async { unreachable!() }
    }

    fn try_send_protocol(&self, _: Self::Protocol) -> Result<(), SendNowError<Self::Protocol>> {
        unreachable!()
    }
}

impl<T: ?Sized> DynAddressSpec for DynSpec<T> {
    fn accepts_msg(&self, msg_type_id: TypeId) -> bool {
        self.spec.accepts_msg_object_safe(msg_type_id)
    }

    async fn send_msg_dyn<M>(&self, msg: M) -> Result<(), SendDynError<M>>
    where
        M: Message + Send + 'static,
    {
        self.spec
            .send_msg_object_safe(Box::new(msg))
            .await
            .map_err(|e| e.downcast::<M>().unwrap_silent())
    }

    fn send_msg_dyn_blocking<M>(&self, msg: M) -> Result<(), SendDynError<M>>
    where
        M: Message + Send + 'static,
    {
        self.spec
            .send_msg_blocking_object_safe(Box::new(msg))
            .map_err(|e| e.downcast::<M>().unwrap_silent())
    }

    fn try_send_msg_dyn<M>(&self, msg: M) -> Result<(), TrySendDynError<M>>
    where
        M: Message + Send + 'static,
    {
        self.spec
            .try_send_msg_object_safe(Box::new(msg))
            .map_err(|e| e.downcast::<M>().unwrap_silent())
    }
}

//-------------------------------------
// DynProtocol
//-------------------------------------

pub struct DynProtocol<D = DynSpec>(AnyBox, PhantomData<D>);

impl<D: ?Sized, M> Protocol<M> for DynProtocol<DynSpec<D>>
where
    DynSpec<D>: ProtocolMarker<M>,
    M: Message + Send + 'static,
{
    fn from_msg(msg: M) -> Self {
        let msg: AnyBox = Box::new(msg);
        Self(msg, PhantomData)
    }

    fn try_into_msg(self) -> Result<M, Self> {
        match self.0.downcast::<M>() {
            Ok(msg) => Ok(*msg),
            Err(msg) => Err(Self(msg, PhantomData)),
        }
    }
}

// impl<D: ?Sized, M> FromMsgMarker<M> for DynSpec<D> where D: FromMsgMarker<M> {}
// impl<D, M> FromMsgMarker<M> for DynProtocol<D> where D: FromMsgMarker<M> {}

// impl<T: ?Sized> FromMsgDyn for DynProtocol<DynSpec<T>> {
//     fn accepted() -> &'static [TypeId] {
//         <DynSpec<T> as FromMsgDyn>::accepted()
//     }

//     fn try_from_boxed_msg(msg: AnyBox) -> Result<Self, AnyBox> {
//         // match msg.downcast::<M>() {
//         //     Ok(msg) => Ok(Self(msg, PhantomData)),
//         //     Err(msg) => Err(msg),
//         // }
//     }

//     fn into_boxed_msg(self) -> AnyBox {
//         self.0
//     }
// }

//-------------------------------------
// AcceptDynObjectSafe
//-------------------------------------

trait AcceptDynObjectSafe: Send + Sync + 'static {
    fn accepts_msg_object_safe(&self, msg_type_id: TypeId) -> bool;
    fn send_msg_object_safe(&self, protocol: AnyBox)
        -> BoxFuture<Result<(), SendDynError<AnyBox>>>;
    fn send_msg_blocking_object_safe(&self, protocol: AnyBox) -> Result<(), SendDynError<AnyBox>>;
    fn try_send_msg_object_safe(&self, payload: AnyBox) -> Result<(), TrySendDynError<AnyBox>>;
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<S> AcceptDynObjectSafe for S
where
    S: AddressSpec + Send + Sync + 'static,
    S::Protocol: DynamicProtocol,
{
    fn accepts_msg_object_safe(&self, msg_type_id: TypeId) -> bool {
        <S::Protocol as DynamicProtocol>::accepts(msg_type_id)
    }

    fn send_msg_object_safe(&self, msg: AnyBox) -> BoxFuture<Result<(), SendDynError<AnyBox>>> {
        Box::pin(async move {
            let protocol = <S::Protocol as DynamicProtocol>::try_from_boxed_msg(msg)
                .map_err(SendDynError::NotAccepted)?;

            self.send_protocol(protocol).await.map_err(|e| match e {
                SendError(protocol) => {
                    SendDynError::Closed(<S::Protocol as DynamicProtocol>::into_boxed_msg(protocol))
                }
            })
        })
    }

    fn send_msg_blocking_object_safe(&self, msg: AnyBox) -> Result<(), SendDynError<AnyBox>> {
        let protocol = <S::Protocol as DynamicProtocol>::try_from_boxed_msg(msg)
            .map_err(SendDynError::NotAccepted)?;

        self.send_protocol_blocking(protocol).map_err(|e| match e {
            SendError(protocol) => {
                SendDynError::Closed(<S::Protocol as DynamicProtocol>::into_boxed_msg(protocol))
            }
        })
    }

    fn try_send_msg_object_safe(&self, msg: AnyBox) -> Result<(), TrySendDynError<AnyBox>> {
        let protocol = <S::Protocol as DynamicProtocol>::try_from_boxed_msg(msg)
            .map_err(TrySendDynError::NotAccepted)?;

        self.try_send_protocol(protocol).map_err(|e| match e {
            SendNowError::Closed(protocol) => {
                TrySendDynError::Closed(<S::Protocol as DynamicProtocol>::into_boxed_msg(protocol))
            }
            SendNowError::Full(protocol) => {
                TrySendDynError::Full(<S::Protocol as DynamicProtocol>::into_boxed_msg(protocol))
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        todo!()
    }
}

//-------------------------------------
// marker
//-------------------------------------

pub mod marker {
    use super::*;
    use crate::message::ProtocolMarker;

    macro_rules! create_markers {
        ($(
            $accepts:ident<$($gen:ident),*> $(:)? $($prev_accept:path),*;
        )*) => {
            $(
                /// A marker trait to indicate that a process accepts a message.
                pub trait $accepts<$($gen),*>: $($prev_accept +)* {}

                // Make the marker-traits auto-traits.
                impl<$($gen,)* S> $accepts<$($gen),*> for S where S: $($prev_accept +)* {}

                // And implement the correct FromSpec implementations
                impl<$($gen,)* S> FromSpec<S> for crate::Spec!($($gen,)*)
                where
                    S: AddressSpec + Send + Sync + 'static,
                    S::Protocol: DynamicProtocol + $accepts<$($gen),*>
                {
                    fn from_spec(spec: S) -> Self {
                        Self {
                            spec: Box::new(spec),
                            t: PhantomData,
                        }
                    }
                }


                // impl<$($gen,)* D: ?Sized> FromSpec<DynSpec<D>> for crate::Protocol!($($gen,)*)
                // where
                //     // S: AddressSpec + Send + Sync + 'static,
                //     // S::Protocol: AcceptsDyn,
                //     D: $accepts<$($gen),*>
                // {
                //     fn from_spec(spec: DynSpec<D>) -> Self {
                //         Self {
                //             spec: spec.spec,
                //             t: PhantomData,
                //         }
                //     }
                // }
            )*
        };
    }

    create_markers!(
        AcceptsNone<>;
        AcceptsOne<T1>: AcceptsNone, ProtocolMarker<T1>;
        AcceptsTwo<T1, T2>: AcceptsOne<T1>, ProtocolMarker<T2>;
        AcceptsThree<T1, T2, T3>: AcceptsTwo<T1, T2>, ProtocolMarker<T3>;
        AcceptsFour<T1, T2, T3, T4>: AcceptsThree<T1, T2, T3>, ProtocolMarker<T4>;
        AcceptsFive<T1, T2, T3, T4, T5>: AcceptsFour<T1, T2, T3, T4>, ProtocolMarker<T5>;
        AcceptsSix<T1, T2, T3, T4, T5, T6>: AcceptsFive<T1, T2, T3, T4, T5>, ProtocolMarker<T6>;
        AcceptsSeven<T1, T2, T3, T4, T5, T6, T7>: AcceptsSix<T1, T2, T3, T4, T5, T6>, ProtocolMarker<T7>;
        AcceptsEight<T1, T2, T3, T4, T5, T6, T7, T8>: AcceptsSeven<T1, T2, T3, T4, T5, T6, T7>, ProtocolMarker<T8>;
    );

    #[macro_export]
    macro_rules! Spec {
        ($(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsNone>
        };
        ($t1:ty $(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsOne<$t1>>
        };
        ($t1:ty, $t2:ty $(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsTwo<$t1, $t2>>
        };
        ($t1:ty, $t2:ty, $t3:ty $(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsThree<$t1, $t2, $t3>>
        };
        ($t1:ty, $t2:ty, $t3:ty, $t4:ty $(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsFour<$t1, $t2, $t3, $t4>>
        };
        ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty $(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsFive<$t1, $t2, $t3, $t4, $t5>>
        };
        ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty, $t6:ty $(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsSix<$t1, $t2, $t3, $t4, $t5, $t6>>
        };
        ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty, $t6:ty, $t7:ty $(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsSeven<$t1, $t2, $t3, $t4, $t5, $t6, $t7>>
        };
        ($t1:ty, $t2:ty, $t3:ty, $t4:ty, $t5:ty, $t6:ty, $t7:ty, $t8:ty $(,)?) => {
            $crate::dynamic::DynSpec<dyn $crate::dynamic::marker::AcceptsEight<$t1, $t2, $t3, $t4, $t5, $t6, $t7, $t8>>
        };
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::dynamic::DynSpec;
        use std::marker::PhantomData;

        #[allow(clippy::type_complexity, unused)]
        fn compilation_test<T1, T2, T3, T4, T5, T6, T7, T8>() {
            let _: PhantomData<Spec!()> = PhantomData::<DynSpec<dyn AcceptsNone>>;
            let _: PhantomData<Spec!(T1)> = PhantomData::<DynSpec<dyn AcceptsOne<T1>>>;
            let _: PhantomData<Spec!(T1, T2)> = PhantomData::<DynSpec<dyn AcceptsTwo<T1, T2>>>;
            let _: PhantomData<Spec!(T1, T2, T3)> =
                PhantomData::<DynSpec<dyn AcceptsThree<T1, T2, T3>>>;
            let _: PhantomData<Spec!(T1, T2, T3, T4)> =
                PhantomData::<DynSpec<dyn AcceptsFour<T1, T2, T3, T4>>>;
            let _: PhantomData<Spec!(T1, T2, T3, T4, T5)> =
                PhantomData::<DynSpec<dyn AcceptsFive<T1, T2, T3, T4, T5>>>;
            let _: PhantomData<Spec!(T1, T2, T3, T4, T5, T6)> =
                PhantomData::<DynSpec<dyn AcceptsSix<T1, T2, T3, T4, T5, T6>>>;
            let _: PhantomData<Spec!(T1, T2, T3, T4, T5, T6, T7)> =
                PhantomData::<DynSpec<dyn AcceptsSeven<T1, T2, T3, T4, T5, T6, T7>>>;
            let _: PhantomData<Spec!(T1, T2, T3, T4, T5, T6, T7, T8)> =
                PhantomData::<DynSpec<dyn AcceptsEight<T1, T2, T3, T4, T5, T6, T7, T8>>>;
        }
    }
}
