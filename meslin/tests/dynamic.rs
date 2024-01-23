use meslin::*;

/// Example protocol that can be used
#[derive(Debug, From, TryInto, DynProtocol)]
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

    let dyn_sender: DynSender![u32, HelloWorld] = sender.clone().into_sender();
    do_something(sender.clone()).await;
    do_something(dyn_sender).await;

    sender.send::<HelloWorld>("Hello world!").await.unwrap();
    sender.send::<HelloWorld>("Hello world!").now().unwrap();
    sender.send::<HelloWorld>("Hello world!").wait().unwrap();
    sender
        .send::<HelloWorld>("Hello world!")
        .with(())
        .await
        .unwrap();
    sender
        .send::<HelloWorld>("Hello world!")
        .with(())
        .now()
        .unwrap();
    sender
        .send::<HelloWorld>("Hello world!")
        .with(())
        .wait()
        .unwrap();
    sender
        .send::<HelloWorld>("Hello world!")
        .with(())
        .wait()
        .unwrap();

    let boxed_sender = sender.clone().boxed();
    boxed_sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    let dyn_sender = <DynSender![HelloWorld]>::from_inner_unchecked(boxed_sender);
    dyn_sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    let dyn_sender = <DynSender![HelloWorld, u32]>::new(sender.clone());
    let dyn_sender = dyn_sender.transform::<Set![u32]>();

    let dyn_sender = dyn_sender.try_transform::<Set![HelloWorld]>().unwrap();
    dyn_sender.try_transform::<Set![u64, u32]>().unwrap_err();
}

async fn do_something(sender: impl IntoSender<DynSender![HelloWorld]>) {
    let sender = sender.into_sender();
    sender.send::<HelloWorld>("Hello world!").await.unwrap();
}
