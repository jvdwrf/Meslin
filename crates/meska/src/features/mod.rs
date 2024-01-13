#[cfg(feature = "mpsc")]
pub mod mpsc;

#[cfg(feature = "request")]
pub mod request;

#[cfg(feature = "derive")]
pub use meska_derive::*;

#[cfg(feature = "broadcast")]
pub mod broadcast;

#[cfg(feature = "watch")]
pub mod watch {
    use crate::*;
    use tokio::sync::watch;

    pub struct Sender<P> {
        sender: watch::Sender<P>,
    }

    impl<P> Sender<P> {
        pub fn inner(&self) -> &watch::Sender<P> {
            &self.sender
        }

        pub fn into_inner(self) -> watch::Sender<P> {
            self.sender
        }

        pub fn from_inner(sender: watch::Sender<P>) -> Self {
            Self { sender }
        }
    }

    impl<P> SendProtocol for Sender<P>
    where
        P: Send + Clone + Sync,
    {
        type Protocol = P;

        async fn send_protocol(
            &self,
            protocol: Self::Protocol,
        ) -> Result<(), Closed<Self::Protocol>> {
            self.sender.send(protocol).map_err(|e| Closed(e.0))
        }
    }

    impl<P> TrySendProtocol for Sender<P>
    where
        P: Send + Clone + Sync,
    {
        type Protocol = P;

        fn try_send_protocol(
            &self,
            protocol: Self::Protocol,
        ) -> Result<(), TrySendError<Self::Protocol>> {
            self.sender
                .send(protocol)
                .map_err(|e| TrySendError::Closed(e.0))
        }
    }
}
