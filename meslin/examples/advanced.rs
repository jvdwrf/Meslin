use meslin::*;

#[derive(Debug, From, TryInto, DynProtocol)]
enum Protocol0 {
    A(i16),
    B(i32),
    C(i64),
}

#[derive(Debug, From, TryInto, DynProtocol)]
enum Protocol1 {
    A(i32),
    B(i64),
    C(i128),
}

#[tokio::main]
async fn main() {
    // Create two different senders with different protocols:
    let (sender0, _receiver0) = mpmc::unbounded::<Protocol0>();
    let (sender1, _receiver1) = priority::unbounded::<Protocol1, u32>();

    // We can send messages normally:
    sender0.send::<i32>(8).await.unwrap();
    sender1.send::<i32>(8).with(15).await.unwrap(); // Uses `15` as priority
    sender1.send::<i32>(8).await.unwrap(); // Uses `u32::default()` as priority

    // Or we can map the senders to dynamic senders: (Compile time checked)
    let senders: Vec<DynSender![i32, i64]> = vec![
        // For sender1, use `into_dyn` to transform it into a DynSender
        sender0.into_sender(),
        // For sender2, use `with` and then `into_dyn` to transform it into a DynSender
        // This sender will always send `15` as the priority
        sender1.with(15).into_sender(),
    ];


    // We can send messages like before:
    senders[0].send::<i32>(8).await.unwrap();
    senders[1].send::<i32>(8).await.unwrap();

    // The following doesn't compile, even though sender1 can send i128...
    // senders[1].send::<i128>(8).await;
    // senders[0].send::<i128>(8).await;

    // ...so we can use `dynamic()` instead:
    senders[0].send::<i128>(8).dynamic().await.unwrap_err(); // <- Runtime error!
    senders[1].send::<i128>(8).dynamic().await.unwrap();     // <- Okay!


    // We can still request basic information about the senders...
    assert_eq!(senders[0].len(), 2); // ...like the amount of messages
    assert_eq!(senders[1].len(), 4);
    assert_eq!(senders[0].capacity(), None); // ...or the capacity
    assert_eq!(senders[1].capacity(), None);

    // Finally, the senders can be converted back into their original types:
    let _sender1 = senders[0]
        .downcast_ref::<mpmc::Sender<Protocol0>>()
        .unwrap();
    let _sender2 = senders[1]
        .downcast_ref::<WithValueSender<priority::Sender<Protocol1, u32>>>()
        .unwrap();
}
