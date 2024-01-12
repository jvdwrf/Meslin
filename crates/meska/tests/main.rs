use crate::common::{HelloWorld, MyProtocol};
use meska::{request::Request, *};
mod common;

#[tokio::test]
async fn test_basic_protocol_sending() {
    let (sender, mut receiver) = mpsc::channel::<MyProtocol>(10);

    sender.send_protocol(MyProtocol::A(1)).await.unwrap();
    sender
        .send_protocol(MyProtocol::B(HelloWorld("hello".to_string())))
        .await
        .unwrap();
    let (request, _rx) = Request::new(10);
    sender.try_send_protocol(MyProtocol::C(request)).unwrap();
    sender.send_protocol_blocking(MyProtocol::A(4)).unwrap();
    drop(sender);

    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::A(1)));
    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::B(_)));
    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::C(_)));
    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::A(4)));
    assert!(receiver.recv().await.is_none());
}

#[tokio::test]
async fn test_basic_msg_sending() {
    let (sender, mut receiver) = mpsc::channel::<MyProtocol>(10);

    sender.send_msg(1u32).await.unwrap();
    sender
        .send_msg(HelloWorld("hello".to_string()))
        .await
        .unwrap();
    let (request, _rx) = Request::new(10);
    sender.try_send_msg(request).unwrap();
    drop(sender);

    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::A(1)));
    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::B(_)));
    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::C(_)));
}

#[tokio::test]
async fn test_basic_sending() {
    let (sender, mut receiver) = mpsc::channel::<MyProtocol>(10);

    sender.send::<u32>(1u32).await.unwrap();
    sender.send::<HelloWorld>("hello").await.unwrap();
    let _rx = sender.try_send::<Request<u32, String>>(10u32).unwrap();

    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::A(1)));
    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::B(_)));
    assert!(matches!(receiver.recv().await.unwrap(), MyProtocol::C(_)));
}
