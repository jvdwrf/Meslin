use meslin::*;

#[derive(Debug, From, TryInto, DynProtocol)]
enum Protocol1 {
    A(i16),
    B(i32),
    C(i64),
}

#[derive(Debug, From, TryInto, DynProtocol)]
enum Protocol2 {
    A(i32),
    B(i64),
    C(i128),
}

#[tokio::main]
async fn main() {
    // Create two different senders with different protocols:
    let (sender1, _receiver1) = mpmc::unbounded::<Protocol1>();
    let (sender2, _receiver2) = priority::unbounded::<Protocol2, u32>();

    // We can send messages normally:
    sender1.send::<i32>(8).await.unwrap();
    sender2.send::<i32>(8).with(15).await.unwrap(); // Uses `15` as priority
    sender2.send::<i32>(8).await.unwrap(); // Uses `u32::default()` as priority

    // Or we can map the senders to dynamic senders: (Compile time checked)
    let senders: Vec<DynSender![i32, i64]> = vec![
        // For sender1, use `into_dyn` to transform it into a DynSender
        sender1.into_sender(),
        // For sender2, use `with` and then `into_dyn` to transform it into a DynSender
        // This sender will always send `15` as the priority
        sender2.with(15).into_sender(),
    ];

    // And then send messages:
    senders[0].send::<i32>(8).await.unwrap();
    senders[1].send::<i32>(8).await.unwrap();
    // dyn_senders[0].send::<i16>(8).await.unwrap(); // <- Doesn't compile!

    // We can still find basic information about the senders:
    assert_eq!(senders[0].len(), 2);
    assert_eq!(senders[1].len(), 3);
    assert_eq!(senders[0].capacity(), None);
    assert_eq!(senders[1].capacity(), None);

    // We can also still find out whether messages are accepted:
    assert!(senders[0].accepts::<i16>());
    assert!(!senders[0].accepts::<i128>());
    assert!(senders[1].accepts::<i128>());
    assert!(!senders[1].accepts::<i16>());

    // Which means we can use the `dyn_send` methods.
    // Instead of not compiling, these methods return an error:
    senders[0].send::<i16>(8i16).dynamic().await.unwrap();
    senders[0].send::<i128>(8i128).dynamic().await.unwrap_err(); // <- Runtime error!
    senders[1].send::<i128>(8i128).dynamic().await.unwrap();
    senders[1].send::<i16>(8i16).dynamic().await.unwrap_err(); // <- Runtime error!

    // And the senders can be converted back into their original types:
    let _sender1 = senders[0]
        .downcast_ref::<mpmc::Sender<Protocol1>>()
        .unwrap();
    let _sender2 = senders[1]
        .downcast_ref::<WithValueSender<priority::Sender<Protocol2, u32>>>()
        .unwrap();
}
