# Meslin: Simplifying Actor-Systems
Meslin is a Rust library offering ergonomic wrappers for channels like `mpmc` and `broadcast`. It's designed to ease the creation of actor systems by adding user-friendly features, without being tied to any specific runtime. This makes it compatible with various runtimes such as `tokio`, `smol`, or `async-std`.

## Purpose and Design
The primary goal of Meslin is to provide a framework for developing actor systems in Rust. It intentionally steers clear of incorporating supervisory functions or other complex features, focusing instead on simplicity and non-interference.

### Core Components
Meslin is built around three key elements:
1. **Messages**: Defined with input and output parameters, messages facilitate sending operations using only the input. This design promotes the creation of messages that expect responses, enhancing their usability.
2. **Protocols**: An actor in Meslin establishes a protocol. This protocol, detailing the acceptable message types, is defined by implementing `From<M>` and `TryInto<M>` traits.
3. **Senders**: These are responsible for defining the delivery mechanism of a protocol to the actor.

### Flexibility and Extensibility
One of the library's strengths lies in its separation of concerns. This separation not only streamlines the development process but also enables the easy integration and customization of messages, protocols, and senders. For instance, swapping an `mpmc` channel for a `priority` channel is straightforward.

### Dynamic Senders
A unique feature of Meslin is the transformation of senders into dynamic senders. This process converts the sender into a trait-object, facilitating the storage of different sender types in the same data structure, like `Vec<T>`. For example, if you have an `mpmc::Sender<ProtocolA>` and a `broadcast::Sender<ProtocolB>`, both accepting messages `Msg1` and `Msg2`, they can be converted into `DynSender<Accepts![Msg1, Msg2]>`. This dynamic sender then implements `Sends<Msg1> + Sends<Msg2>`, allowing for versatile storage solutions.

### Zero-cost
Meslin is designed with a zero-cost abstraction principle in mind, ensuring that its ease of use and flexibility don't compromise performance. When not using any dynamic features of the library, Meslin does not add any additional runtime overhead compared to hand-written equivalents.

## Cargo features
- Default: `["derive", "request", "mpmc", "broadcast", "priority"]`
- Non-default: `["watch"]`

## Basic example
```rust
use meslin::{mpmc, From, Message, Request, SendsExt, TryInto};

// Create a simple, custom message type
#[derive(Debug, From, Message)]
#[from(forward)]
struct MyMessage(String);

// Create the protocol used by the actor
// It defines the messages that can be sent
#[derive(Debug, From, TryInto)]
enum MyProtocol {
    Number(i32),
    Message(MyMessage),
    Request(Request<i32, String>),
}

#[tokio::main]
async fn main() {
    // Create the channel and spawn a task that receives messages
    let (sender, receiver) = mpmc::unbounded::<MyProtocol>();
    tokio::task::spawn(receive_messages(receiver));

    // Send a number
    sender.send::<i32>(42).await.unwrap();

    // Send a message
    sender.send::<MyMessage>("Hello").await.unwrap();

    // Send a request and then wait for the reply (oneshot channel)
    let rx = sender.send::<Request<i32, String>>(42).await.unwrap();
    let reply = rx.await.unwrap();
    assert_eq!(reply, "The number is 42");

    // Send a request and receive the reply immeadiately
    let reply = sender.request::<Request<i32, String>>(42).await.unwrap();
    assert_eq!(reply, "The number is 42");
}

// This is completely standard: `mpmc::Receiver` == `flume::Receiver`
async fn receive_messages(receiver: mpmc::Receiver<MyProtocol>) {
    while let Ok(msg) = receiver.recv_async().await {
        match msg {
            MyProtocol::Number(msg) => {
                println!("Received number: {msg:?}");
            }
            MyProtocol::Message(msg) => {
                println!("Received message: {msg:?}");
            }
            MyProtocol::Request(Request { msg, tx }) => {
                println!("Received request: {msg:?}");
                tx.send(format!("The number is {}", msg)).ok();
            }
        }
    }
}
```

## Advanced example
```rust
use meslin::{
    mpmc, priority, Accepts, DynFromInto, DynSender, DynSendsExt, From, MappedWithSender, SendsExt,
    TryInto, WithValueSender,
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
    let _sender3 = senders[2]
        .downcast_ref::<MappedWithSender<priority::Sender<P2, u32>, ()>>()
        .unwrap();
}
```

