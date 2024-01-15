#![allow(clippy::type_complexity)]

mod features;
mod message;
mod protocol;
mod sending;
mod dynamic;
mod errors;


pub use errors::*;
pub use features::*;
pub use message::*;
pub use protocol::*;
pub use sending::*;
pub use dynamic::*;

type AnyBox = Box<dyn std::any::Any + Send + 'static>;
type BoxError = Box<dyn std::error::Error + Send>;

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
