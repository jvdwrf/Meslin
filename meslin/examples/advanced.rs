use meslin::{
    mpmc, priority, Accepts, DynFromInto, DynSender, DynSendsExt, From, SendsExt, TryInto,
    WithValueSender,
};

#[derive(Debug, From, TryInto, DynFromInto)]
enum P1 {
    A(i32),
    B(i64),
    C(i128),
}

#[derive(Debug, From, TryInto, DynFromInto)]
enum P2 {
    A(i16),
    B(i32),
    C(i64),
}

#[tokio::main]
async fn main() {
    // Create two different senders, sending different protocols
    let (sender1, _receiver) = mpmc::unbounded::<P1>(); // Sends `P1` with `()`
    let (sender2, _receiver) = priority::unbounded::<P2, u32>(); // Sends `P2` with `u32` as priority

    // Sending messages to the senders:
    sender1.send::<i32>(8).await.unwrap(); // Normal
    sender2.send::<i32>(8).await.unwrap(); // Uses `u32::default()` as priority
    sender2.send_with::<i32>(8, 15).await.unwrap(); // Uses `15` as priority

    // Create a vector of dynamic senders: (Checked at compile time)
    let senders: Vec<DynSender<Accepts![i32, i64]>> = vec![
        // For sender1, use `into_dyn` to transform it into a DynSender
        sender1.into_dyn(),
        // For sender2, use `with` / `map_with` and then `into_dyn` to transform it into a DynSender
        // This sender will always send `15` as the priority
        sender2.clone().with(15).into_dyn(),
        sender2.map_with(|_| 15, |_| ()).into_dyn(),
    ];

    // Send a `i32` or `i64` to the senders
    senders[0].send::<i32>(8).await.unwrap();
    senders[1].send::<i64>(8).await.unwrap();
    senders[2].send::<i32>(8).await.unwrap();

    // Downcast the senders back to their original types
    let _sender1 = senders[0].downcast_ref::<mpmc::Sender<P1>>().unwrap();
    let _sender2 = senders[1]
        .downcast_ref::<WithValueSender<priority::Sender<P2, u32>, ()>>()
        .unwrap();
    // let _sender3 = senders[2].downcast_ref::<???>().unwrap(); -> Unnameable type
}
