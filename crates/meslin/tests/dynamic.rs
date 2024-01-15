use meslin::*;

/// Example protocol that can be used
#[derive(Debug, Message, From, TryInto)]
pub enum MyProtocol {
    A(u32),
    B(HelloWorld),
    C(Request<u32, String>),
}

impl DynFromInto for MyProtocol {
    fn accepts_all() -> &'static [std::any::TypeId] {
        &[]
    }

    fn try_from_boxed_msg<W: 'static>(msg: BoxedMsg<W>) -> Result<(Self, W), BoxedMsg<W>> {
        todo!()
    }

    fn into_boxed_msg<W: Send + 'static>(self, with: W) -> BoxedMsg<W> {
        todo!()
    }
}

#[derive(Debug, Message, From)]
#[from(forward)]
pub struct HelloWorld(pub String);

#[tokio::test]
async fn test() {
    let (sender, _receiver) = mpmc::unbounded::<MyProtocol>();

    let sender = sender.into_boxed_sender();

    sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    let sender = DynSender::<()>::from_inner_unchecked(sender);

    sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    // sender
    //     .send::<HelloWorld>("Hello world!")
    //     .await
    //     .unwrap();

    ()
}
