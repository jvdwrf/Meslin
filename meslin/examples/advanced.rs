use meslin::{mpmc, From, Message, Request, SendsExt, TryInto};

#[derive(Debug, From, Message)]
#[from(forward)]
struct MyMessage(String);

#[derive(Debug, From, TryInto)]
enum MyProtocol {
    Number(i32),
    Message(MyMessage),
    Request(Request<i32, String>),
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpmc::unbounded::<MyProtocol>();
    tokio::task::spawn(mpmc_task(receiver));

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

async fn mpmc_task(receiver: mpmc::Receiver<MyProtocol>) {
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

// async fn priority_task(receiver: priority::Receiver<MyProtocol>) {
//     while let Ok(msg) = receiver.recv_async().await {
//         match msg {
//             MyProtocol::Number(msg) => {
//                 println!("Received number: {msg:?}");
//             }
//             MyProtocol::Message(msg) => {
//                 println!("Received message: {msg:?}");
//             }
//             MyProtocol::Request(Request { msg, tx }) => {
//                 println!("Received request: {msg:?}");
//                 tx.send(format!("The number is {}", msg)).ok();
//             }
//         }
//     }
// }
