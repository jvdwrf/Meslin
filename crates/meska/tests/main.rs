use meska::{request::Request, *};

/// Example protocol that can be used
#[derive(Debug, Protocol)]
pub enum MyProtocol {
    A(u32),
    B(HelloWorld),
    C(Request<u32, String>),
}

#[derive(Debug, Message, From)]
pub struct HelloWorld(pub String);

#[tokio::test]
async fn test_basic_protocol_sending() {
    let (sender, mut receiver) = mpsc::channel::<MyProtocol>(10);

    tokio::task::spawn(async move {
        assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::A(1)));
        assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::B(_)));
        assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::C(_)));
        assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::A(4)));
        assert!(receiver.recv().await.is_none());
    });

    sender.send_protocol(MyProtocol::A(1)).await.unwrap();
    sender
        .send_protocol(MyProtocol::B(HelloWorld("hello".to_string())))
        .await
        .unwrap();
    let (request, _rx) = Request::new(10);
    sender.send_protocol_now(MyProtocol::C(request)).unwrap();
    sender.send_protocol_blocking(MyProtocol::A(4)).unwrap();
    drop(sender);
}

#[tokio::test]
async fn test_basic_msg_sending() {
    let (sender, mut receiver) = mpsc::channel::<MyProtocol>(10);

    tokio::task::spawn(async move {
        assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::A(1)));
        assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::B(_)));
        assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::C(_)));
    });

    sender.send_msg(1u32).await.unwrap();
    sender
        .send_msg(HelloWorld("hello".to_string()))
        .await
        .unwrap();
    let (request, _rx) = Request::new(10);
    sender.send_msg_now(request).unwrap();
    drop(sender);
}

#[tokio::test]
async fn test_basic_sending() {
    let (tx, mut rx) = mpsc::channel::<MyProtocol>(10);

    tokio::task::spawn(async move {
        assert!(matches!(rx.recv().await.unwrap(), MyProtocol::A(1)));
        assert!(matches!(rx.recv().await.unwrap(), MyProtocol::B(_)));
        let MyProtocol::C(Request { msg, tx }) = rx.recv().await.unwrap() else {
            panic!()
        };
        tx.send(format!("Your number was: {msg}")).unwrap();
    });

    tx.send::<u32>(1u32).await.unwrap();
    tx.send::<HelloWorld>("hello").await.unwrap();
    let reply = tx.request::<Request<u32, String>>(10u32).await.unwrap();
    assert_eq!(reply, "Your number was: 10");
}
