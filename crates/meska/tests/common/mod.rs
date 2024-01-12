use meska::{request::Request, *};

/// Example protocol that can be used
#[derive(Debug /* Protocol */)]
pub enum MyProtocol {
    A(u32),
    B(HelloWorld),
    C(Request<u32, String>),
}

#[derive(Debug /* Message */)]
pub struct HelloWorld(pub String);

//-------------------------------------
// Generated code
//-------------------------------------

impl Message for HelloWorld {
    type Input = String;
    type Output = ();

    fn create(from: Self::Input) -> (Self, Self::Output) {
        (Self(from), ())
    }

    fn cancel(self, _: Self::Output) -> Self::Input {
        self.0
    }
}

impl From<String> for HelloWorld {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl ProtocolFor<u32> for MyProtocol {
    fn from_msg(msg: u32) -> Self {
        Self::A(msg)
    }

    fn try_into_msg(self) -> Result<u32, Self> {
        match self {
            Self::A(msg) => Ok(msg),
            _ => Err(self),
        }
    }
}

impl ProtocolFor<HelloWorld> for MyProtocol {
    fn from_msg(msg: HelloWorld) -> Self {
        Self::B(msg)
    }

    fn try_into_msg(self) -> Result<HelloWorld, Self> {
        match self {
            Self::B(msg) => Ok(msg),
            _ => Err(self),
        }
    }
}

impl ProtocolFor<Request<u32, String>> for MyProtocol {
    fn from_msg(msg: Request<u32, String>) -> Self {
        Self::C(msg)
    }

    fn try_into_msg(self) -> Result<Request<u32, String>, Self> {
        match self {
            Self::C(msg) => Ok(msg),
            _ => Err(self),
        }
    }
}
