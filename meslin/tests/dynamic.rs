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

    let _ = sender.clone().into_dyn::<Accepts![u32]>();

    let boxed_sender = sender.clone().into_boxed();
    boxed_sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    let dyn_sender = DynSender::<Accepts![HelloWorld]>::from_boxed_unchecked(boxed_sender);
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

// #[tokio::test]
// async fn test2() {
//     #[derive(Debug, From, TryInto, DynFromInto)]
//     enum P1 {
//         A(u32),
//         B(u64),
//     }

//     #[derive(Debug, From, TryInto, DynFromInto)]
//     enum P2 {
//         A(u16),
//         B(u32),
//     }

//     let (sender1, _receiver) = mpmc::unbounded::<P1>();
//     let (sender2, _receiver) = priority::unbounded::<P2, u32>();

//     let senders /*: Vec<DynSender<Accepts![u32]>> */ = vec![
//         sender1.into_dyn::<Accepts![u32]>(), 
//         sender2.into_dyn_with::<Accepts![u32], ()>(0),
//     ];

//     senders[0].send::<u32>(8u32).await.unwrap();
//     senders[1].send::<u32>(8u32).await.unwrap();

//     let _sender1 = senders[0].downcast_ref::<mpmc::Sender<P1>>().unwrap();
//     let _sender2 = senders[1]
//         .downcast_ref::<WithValueSender<priority::Sender<P2, u32>, ()>>()
//         .unwrap();
// }
