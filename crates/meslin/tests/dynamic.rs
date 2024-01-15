use meslin::*;
use std::any::TypeId;

/// Example protocol that can be used
#[derive(Debug, Message, From, TryInto)]
pub enum MyProtocol {
    A(u32),
    B(HelloWorld),
    C(Request<u32, String>),
}

impl AcceptsAll for MyProtocol {
    fn accepts_all() -> &'static [std::any::TypeId] {
        static LOCK: std::sync::OnceLock<[TypeId; 3]> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| {
            [
                std::any::TypeId::of::<u32>(),
                std::any::TypeId::of::<HelloWorld>(),
                std::any::TypeId::of::<Request<u32, String>>(),
            ]
        })
    }
}

impl DynFromInto for MyProtocol {
    fn try_from_boxed_msg<W: 'static>(
        msg: crate::BoxedMsg<W>,
    ) -> Result<(Self, W), crate::BoxedMsg<W>> {
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

    fn into_boxed_msg<W: Send + 'static>(self, with: W) -> crate::BoxedMsg<W> {
        match self {
            MyProtocol::A(msg) => crate::BoxedMsg::new(msg, with),
            MyProtocol::B(msg) => crate::BoxedMsg::new(msg, with),
            MyProtocol::C(msg) => crate::BoxedMsg::new(msg, with),
        }
    }
}

impl Accepts<u32> for MyProtocol {}
impl Accepts<HelloWorld> for MyProtocol {}
impl Accepts<Request<u32, String>> for MyProtocol {}

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

    let dyn_sender = DynSender::<Accepts![HelloWorld]>::from_inner_unchecked(boxed_sender);
    dyn_sender
        .dyn_send::<HelloWorld>("Hello world!")
        .await
        .unwrap();

    let dyn_sender = DynSender::<Accepts![HelloWorld, u32]>::new(sender.clone());
    let dyn_sender = dyn_sender.transform::<Accepts![u32]>();

    let dyn_sender = dyn_sender.try_transform::<Accepts![HelloWorld]>().unwrap();
    dyn_sender
        .try_transform::<Accepts![u64, u32]>()
        .unwrap_err();
}
