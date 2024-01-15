use std::{any::TypeId, sync::OnceLock};

use meslin::{
    marker::{AcceptsNone, AcceptsOne},
    *,
};

/// Example protocol that can be used
#[derive(Debug, Message, From, TryInto)]
pub enum MyProtocol {
    A(u32),
    B(HelloWorld),
    C(Request<u32, String>),
}

impl DynFromInto for MyProtocol {
    fn accepts_all() -> &'static [std::any::TypeId] {
        static LOCK: OnceLock<[TypeId; 3]> = OnceLock::new();
        LOCK.get_or_init(|| {
            [
                TypeId::of::<u32>(),
                TypeId::of::<HelloWorld>(),
                TypeId::of::<Request<u32, String>>(),
            ]
        })
    }

    fn try_from_boxed_msg<W: 'static>(msg: BoxedMsg<W>) -> Result<(Self, W), BoxedMsg<W>> {
        let msg = match msg.downcast::<u32>() {
            Ok((msg, with)) => return Ok((MyProtocol::A(msg), with)),
            Err(msg) => msg,
        };
        let msg = match msg.downcast::<HelloWorld>() {
            Ok((msg, with)) => return Ok((MyProtocol::B(msg), with)),
            Err(msg) => msg,
        };
        let msg = match msg.downcast::<Request<u32, String>>() {
            Ok((msg, with)) => return Ok((MyProtocol::C(msg), with)),
            Err(msg) => msg,
        };
        Err(msg)
    }

    fn into_boxed_msg<W: Send + 'static>(self, with: W) -> BoxedMsg<W> {
        match self {
            MyProtocol::A(msg) => BoxedMsg::new(msg, with),
            MyProtocol::B(msg) => BoxedMsg::new(msg, with),
            MyProtocol::C(msg) => BoxedMsg::new(msg, with),
        }
    }
}

#[derive(Debug, Message, From)]
#[from(forward)]
pub struct HelloWorld(pub String);

#[tokio::test]
async fn test() {
    let (sender, _receiver) = mpmc::unbounded::<MyProtocol>();

    let boxed_sender = sender.clone().into_boxed_sender();
    boxed_sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    let dyn_sender = DynSender::<Accepts!(HelloWorld)>::from_inner_unchecked(boxed_sender);
    dyn_sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    static LOCK: OnceLock<[TypeId; 1]> = OnceLock::new();
    LOCK.get_or_init(|| [TypeId::of::<HelloWorld>()]);

    let dyn_sender = DynSender::<Accepts![HelloWorld, u32]>::new(sender.clone());
    let dyn_sender = dyn_sender.transform::<Accepts![u32]>();

    let dyn_sender = dyn_sender.try_transform::<Accepts![HelloWorld]>().unwrap();
    dyn_sender.try_transform::<Accepts![u64, u32]>().unwrap_err();
}
