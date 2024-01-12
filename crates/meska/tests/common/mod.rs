use meska::{request::Request, *};

/// Example protocol that can be used
#[derive(Debug, Protocol, DynProtocol)]
pub enum MyProtocol {
    A(u32),
    B(HelloWorld),
    C(Request<u32, String>),
}

#[derive(Debug, Message)]
pub struct HelloWorld(pub String);

impl<T: Into<String>> From<T> for HelloWorld {
    fn from(s: T) -> Self {
        Self(s.into())
    }
}
