

pub struct Inbox<S> {
    inbox: S
}

impl<T> Inbox<T> {
    pub fn from_inner(inner: T) -> Self {
        Self {
            inbox: inner
        }
    }
}