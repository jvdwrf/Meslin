pub mod address;
pub mod child;
pub mod specification;
pub mod tokio_task;
pub mod spawn;
pub mod inbox;
pub mod message;
pub mod dynamic;


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