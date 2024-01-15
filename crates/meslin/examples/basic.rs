use meslin::{mpmc, DynFromInto, From, Message, Request, SendsExt, TryInto};

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

    sender.send::<MyMessage>("Hello").await.unwrap();
    sender.send::<i32>(42).await.unwrap();
    let reply = sender.request::<Request<i32, String>>(42).await.unwrap();
    assert_eq!(reply, "The number is 42");
}

async fn task(receiver: mpmc::Receiver<MyProtocol>) {
    while let Ok(msg) = receiver.recv_async().await {
        match msg {
            MyProtocol::Message(msg) => {
                println!("Received message: {:?}", msg);
            }
            MyProtocol::Number(num) => {
                println!("Received number: {:?}", num);
            }
            MyProtocol::Request(Request { msg, tx }) => {
                println!("Received request: {:?}", msg);
                tx.send(format!("The number is {}", msg)).ok();
            }
        }
    }
}
