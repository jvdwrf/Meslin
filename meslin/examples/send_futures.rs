use meslin::*;

// Create a simple protocol
#[derive(Debug, From, TryInto, DynProtocol)]
enum MyProtocol {
    Number(i32),
    Request(Request<i32, ()>),
}

#[tokio::main]
async fn main() {
    // Create and spawn a simple task that receives messages
    let (sender, receiver) = priority::unbounded::<MyProtocol, u8>();

    tokio::task::spawn(async move {
        while let Ok((protocol, _priority)) = receiver.recv().await {
            match protocol {
                MyProtocol::Number(n) => println!("Received number: {}", n),
                MyProtocol::Request(Request { msg, tx }) => {
                    println!("Received request: {:?}", msg);
                    tx.send(()).unwrap();
                }
            }
        }
    });

    // The most basic send-operation: Waits asynchronously until the message is sent.
    // This uses `u8::default()` as the priority.
    sender.send::<i32>(42).await.unwrap();

    // The `with` method can be used to set the priority of the message.
    // Here we set the priority to `10`.
    sender.send::<i32>(42).with(10).await.unwrap();

    // Instead of using async, we can also block the thread...
    sender.send::<i32>(42).wait().unwrap();
    sender.send::<i32>(42).with(10).wait().unwrap();
    // ... or execute sending with `now`, which succeeds only if space is available now:
    sender.send::<i32>(42).now().unwrap();
    sender.send::<i32>(42).with(10).now().unwrap();

    // We can also send requests...
    sender
        .send::<Request<i32, ()>>(42)
        .await
        .unwrap()
        .await
        .unwrap();
    // ...but this is easier with the `recv()` modifier:
    sender.send::<Request<i32, ()>>(42).recv().await.unwrap();
    sender.send::<Request<i32, ()>>(42).recv().wait().unwrap();

    // `send` accepts the `Message::Input` instead of the message itself.
    // If we write the equivalent using `send_msg`, we get the following:
    // (`send_msg` also accepts modifiers like `with` and `wait`)
    let (request, reply) = Request::new(42);
    sender.send_msg(request).await.unwrap();
    reply.await.unwrap();

    // For the rest of this example, we convert the sender into a dynamic sender.
    // This sender only accepts `i32` messages with `u8` priority.
    let sender: DynSender![i32; u8] = sender.into_sender();

    // We can use it exactly like before:
    sender.send::<i32>(42).await.unwrap();
    sender.send::<i32>(42).with(10).await.unwrap();
    sender.send::<i32>(42).wait().unwrap();

    // Except that the sender only accepts `i32`, and the following doesn't compile:
    // sender.send::<Request<i32, ()>>(42).await; // <- Compile error!

    // To fix this, we can use `dynamic` modifier.
    // This checks at runtime if the message is accepted by the protocol.
    sender.send::<Request<i32, ()>>(42).dynamic().await.unwrap();

    // Sending a message that is not accepted results in an error:
    sender.send::<String>("ERROR").dynamic().await.unwrap_err();

    // As a final example, combining everything together:
    sender
        .send::<Request<i32, ()>>(42)
        .with(10)
        .dynamic()
        .recv()
        .wait()
        .unwrap();
}
