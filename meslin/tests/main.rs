use meslin::*;

/// Example protocol that can be used
#[derive(Debug, Message, From, TryInto)]
pub enum MyProtocol {
    A(u32),
    B(HelloWorld),
    C(Request<u32, String>),
}

#[derive(Debug, Message, From)]
#[from(forward)]
pub struct HelloWorld(pub String);

#[tokio::test]
async fn test_basic_protocol_sending() {
    let (sender, receiver) = mpmc::unbounded::<MyProtocol>();

    tokio::task::spawn(async move {
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::A(1)
        ));
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::B(_)
        ));
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::C(_)
        ));
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::A(4)
        ));
        assert!(receiver.recv_async().await.is_err());
    });

    sender.send::<MyProtocol>(MyProtocol::A(1)).await.unwrap();
    sender
        .send::<MyProtocol>(MyProtocol::B(HelloWorld("hello".to_string())))
        .await
        .unwrap();
    let (request, _rx) = Request::new(10);
    sender
        .try_send::<MyProtocol>(MyProtocol::C(request))
        .unwrap();
    sender.try_send::<MyProtocol>(MyProtocol::A(4)).unwrap();
    drop(sender);
}

#[tokio::test]
async fn test_basic_msg_sending() {
    let (sender, receiver) = mpmc::unbounded::<MyProtocol>();

    tokio::task::spawn(async move {
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::A(1)
        ));
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::B(_)
        ));
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::C(_)
        ));
    });

    sender.send_msg(1u32).await.unwrap();
    sender
        .send_msg(HelloWorld("hello".to_string()))
        .await
        .unwrap();
    let (request, _rx) = Request::new(10);
    sender.try_send_msg(request).unwrap();
    drop(sender);
}

#[tokio::test]
async fn test_basic_sending() {
    let (sender, receiver) = mpmc::unbounded::<MyProtocol>();

    tokio::task::spawn(async move {
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::A(1)
        ));
        assert!(matches!(
            receiver.recv_async().await.unwrap(),
            MyProtocol::B(_)
        ));
        let MyProtocol::C(Request { msg, tx }) = receiver.recv_async().await.unwrap() else {
            unreachable!()
        };
        tx.send(format!("Your number was: {msg}")).unwrap();
    });

    sender.send::<u32>(1u32).await.unwrap();
    sender.send::<HelloWorld>("hello").await.unwrap();
    let reply = sender
        .send::<Request<u32, String>>(10u32)
        .await
        .unwrap()
        .await
        .unwrap();
    assert_eq!(reply, "Your number was: 10");
}

#[tokio::test]
async fn priority() {
    let (tx, rx) = priority::unbounded::<MyProtocol, u32>();

    tx.send::<u32>(0u32).await.unwrap();
    tx.send_with::<u32>(1u32, 2).await.unwrap();
    tx.send_with::<HelloWorld>("hello", 3).await.unwrap();

    assert!(matches!(rx.recv().await.unwrap(), (MyProtocol::B(_), _)));
    assert!(matches!(rx.recv().await.unwrap(), (MyProtocol::A(1), _)));
    assert!(matches!(rx.recv().await.unwrap(), (MyProtocol::A(0), _)));
}
