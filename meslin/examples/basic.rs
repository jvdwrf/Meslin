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
