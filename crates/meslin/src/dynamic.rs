use crate::*;
use futures::{future::BoxFuture, Future};
use std::any::TypeId;

// / DynSender![Ping, Pong; u32]
pub struct DynSender<T: ?Sized> {
    sender: Box<T>,
}

pub trait DynSpecifier {
    type With;
}

#[allow(clippy::len_without_is_empty)]
pub trait DynSendMessage<W = ()>: IsSender<W> {
    fn try_send_msg_with<M>(
        &self,
        msg: M,
        with: W,
    ) -> Result<impl Future<Output = Result<(), Error<(M, W), BoxError>>> + Send, (M, W)>
    where
        M: Send + 'static,
        W: Send + 'static,
        Self: Sized,
    {
        match self._try_send_msg_with(Box::new((msg, with))) {
            Ok(send_future) => Ok(async {
                match send_future.await {
                    Ok(()) => Ok(()),
                    Err(e) => Err(*e.downcast::<Error<(M, W), BoxError>>().unwrap_silent()),
                }
            }),
            Err(not_accepted) => Err(*not_accepted.downcast::<(M, W)>().unwrap_silent()),
        }
    }

    /// Arguments:
    /// - input: `(Message, With)`
    /// - output `AnyBox`: `Error<(Message, With), BoxError>`
    fn _try_send_msg_with(&self, msg_with: AnyBox)
        -> Result<BoxFuture<Result<(), AnyBox>>, AnyBox>;

    fn accepts<M: 'static>(&self) -> bool
    where
        Self: Sized,
    {
        self.accepts_all().contains(&TypeId::of::<M>())
    }

    fn accepts_all(&self) -> &'static [TypeId];
}
