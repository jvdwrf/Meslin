#![allow(clippy::type_complexity)]

mod dynamic;
mod errors;
mod features;
mod message;
mod sending;

pub use dynamic::*;
pub use errors::*;
pub use features::*;
pub use message::*;
pub use sending::*;

type AnyBox = Box<dyn std::any::Any + Send + 'static>;

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
