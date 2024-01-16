# Meslin: Simplifying Actor-System Creation

Meslin is a Rust library offering ergonomic wrappers for channels like `mpmc` and `broadcast`. It's designed to ease the creation of actor systems by adding user-friendly features, without being tied to any specific runtime. This makes it compatible with various runtimes such as `tokio`, `smol`, or `async-std`.

## Purpose and Design
The primary goal of Meslin is to provide a foundational framework for developing actor systems in Rust. It intentionally steers clear of incorporating supervisory functions or other complex features, focusing instead on simplicity and non-interference.

### Core Components
Meslin is built around three key elements:
1. **Messages**: Defined with input and output parameters, messages facilitate sending operations using only the input. This design promotes the creation of messages that expect responses, enhancing their usability.
2. **Protocols**: An actor in Meslin establishes a protocol. This protocol, detailing the acceptable message types, is defined by implementing `From<M>` and `TryInto<M>` traits.
3. **Senders**: These are responsible for defining the delivery mechanism of a protocol to the actor.

### Flexibility and Extensibility
One of the library's strengths lies in its separation of concerns. This separation not only streamlines the development process but also enables the easy integration and customization of messages, protocols, and senders. For instance, swapping an `mpmc` channel for a `priority` channel is straightforward.

### Dynamic Senders
A unique feature of Meslin is the transformation of senders into dynamic senders (`DynSender`). This process converts the sender into a trait-object, facilitating the storage of different sender types in the same data structure, like `Vec<T>`. For example, if you have an `mpmc::Sender<ProtocolA>` and a `broadcast::Sender<ProtocolB>`, both accepting messages `Msg1` and `Msg2`, they can be converted into `DynSender<Accepts![Msg1, Msg2]>`. This dynamic sender then implements `Sends<Msg1> + Sends<Msg2>`, allowing for versatile storage solutions.

## Features
- Default: `["derive", "request", "mpmc", "broadcast", "priority"]`
- Non-default: `["watch"]`

## Basic example
```rust
use meslin::{mpmc, From, Message, Request, SendsExt, TryInto};

#[derive(Debug, From, Message)]
#[from(forward)]
struct MyMessage(String);

#[derive(Debug, From, TryInto)]
enum MyProtocol {
    Message(MyMessage),
    Number(i32),
    Request(Request<i32, String>),
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpmc::unbounded::<MyProtocol>();
    tokio::task::spawn(task(receiver));

    // Send a message
    sender.send::<MyMessage>("Hello").await.unwrap();

    // Send a number
    sender.send::<i32>(42).await.unwrap();

    // Send a request and then wait for the reply (oneshot channel)
    let rx = sender.send::<Request<i32, String>>(42).await.unwrap();
    let reply = rx.await.unwrap();
    assert_eq!(reply, "The number is 42");

    // Send a request and receive the reply immeadiately
    let reply = sender.request::<Request<i32, String>>(42).await.unwrap();
    assert_eq!(reply, "The number is 42");
}

async fn task(receiver: mpmc::Receiver<MyProtocol>) {
    while let Ok(msg) = receiver.recv_async().await {
        match msg {
            MyProtocol::Message(msg) => {
                println!("Received message: {msg:?}");
            }
            MyProtocol::Number(msg) => {
                println!("Received number: {msg:?}");
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

```

