use crate::specification::ChildSpec;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Child<S> {
    t: PhantomData<S>,
    child: S,
}

impl<S: ChildSpec> Unpin for Child<S> {}

impl<S: ChildSpec> Child<S> {
    pub(crate) fn from_inner(inner: S) -> Self {
        Self {
            t: PhantomData,
            child: inner,
        }
    }
}
