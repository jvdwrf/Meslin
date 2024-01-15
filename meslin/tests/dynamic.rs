use meslin::*;

/// Example protocol that can be used
#[derive(Debug, From, TryInto, DynFromInto)]
pub enum MyProtocol {
    A(u32),
    B(HelloWorld),
    C(Request<u32, String>),
}

#[derive(Debug, Message, From)]
#[from(forward)]
pub struct HelloWorld(pub String);

#[tokio::test]
async fn test() {
    let (sender, _receiver) = mpmc::unbounded::<MyProtocol>();

    let boxed_sender = sender.clone().into_boxed_sender();
    boxed_sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    let dyn_sender = DynSender::<Accepts![HelloWorld]>::from_boxed_sender_unchecked(boxed_sender);
    dyn_sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    let dyn_sender = DynSender::<Accepts![HelloWorld, u32]>::new(sender.clone());
    let dyn_sender = dyn_sender.transform::<Accepts![u32]>();

    let dyn_sender = dyn_sender.try_transform::<Accepts![HelloWorld]>().unwrap();
    dyn_sender
        .try_transform::<Accepts![u64, u32]>()
        .unwrap_err();
}
