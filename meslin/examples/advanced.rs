use meslin::{
    mpmc, priority, Accepts, DynFromInto, DynSendsExt, From, SendsExt, TryInto, WithValueSender,
};

#[derive(Debug, From, TryInto, DynFromInto)]
enum P1 {
    A(u32),
    B(u64),
}

#[derive(Debug, From, TryInto, DynFromInto)]
enum P2 {
    A(u16),
    B(u32),
}

#[tokio::main]
async fn main() {
    // Create two different senders, sending different protocols
    let (sender1, _receiver) = mpmc::unbounded::<P1>(); // Sends `P1` with `()`
    let (sender2, _receiver) = priority::unbounded::<P2, u32>(); // Sends `P2` with `u32` as priority

    // Sending messages to the senders:
    sender1.send::<u32>(8u32).await.unwrap(); // Normal
    sender2.send::<u32>(8u32).await.unwrap(); // Uses `u32::default()` as priority
    sender2.send_with::<u32>(8u32, 15).await.unwrap(); // Uses `15` as priority

    // Now we can create a vector of dynamic senders:
    let senders = vec![
        // For sender1, we can use `into_dyn` to transform it into a DynSender
        sender1.into_dyn::<Accepts![u32]>(),
        // For sender2, we can use `with` and then `into_dyn` to transform it into a DynSender
        // This sender will always send `15` as the priority
        sender2.clone().with(15).into_dyn::<Accepts![u32]>(),
        // We can also use `map_with` to transform the sender
        sender2
            .map_with(|_: ()| 15, |_: u32| ())
            .into_dyn::<Accepts![u32]>(),
    ];

    // Now we can send a `u32` to both senders
    senders[0].send::<u32>(8u32).await.unwrap();
    senders[1].send::<u32>(8u32).await.unwrap();
    senders[2].send::<u32>(8u32).await.unwrap();

    // We can also downcast the senders back to their original types
    let _sender1 = senders[0].downcast_ref::<mpmc::Sender<P1>>().unwrap();
    let _sender2 = senders[1]
        .downcast_ref::<WithValueSender<priority::Sender<P2, u32>, ()>>()
        .unwrap();
    // let _sender3 = senders[2].downcast_ref::<???>().unwrap(); -> Unnameable type
}
