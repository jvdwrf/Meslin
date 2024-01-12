use futures::Future;

use crate::{
    address::Address,
    child::Child,
    inbox::Inbox,
    specification::{AddressSpec, ChannelSpec, ChildSpec, InboxSpec},
};

pub fn spawn<Chld, Chnl, Fun, Fut>(
    child_cfg: Option<Chld::Config>,
    channel_cfg: Option<Chnl::Config>,
    f: Fun,
) -> (Child<Chld>, Address<Chnl::AddressSpec>)
where
    Fun: FnOnce(Inbox<Chnl::InboxSpec>, Address<Chnl::AddressSpec>) -> Fut + Send + 'static,
    Fut: Future<Output = Chld::Output> + Send + 'static,
    Fut::Output: Send + 'static,
    Chld: ChildSpec,
    Chnl: ChannelSpec,
    Chnl::AddressSpec: Clone + Send + 'static,
    Chnl::InboxSpec: Send + 'static,
{
    let (ibx, adr) = Chnl::create(channel_cfg);
    let inbox = Inbox::from_inner(ibx);
    let address = Address::from_inner(adr);
    let address2 = address.clone();
    let chd = Chld::spawn_future(child_cfg, async move { f(inbox, address2).await });
    (Child::from_inner(chd), address)
}

pub fn spawn_default<Chld, Chnl, Fun, Fut>(f: Fun) -> (Child<Chld>, Address<Chnl::AddressSpec>)
where
    Fun: FnOnce(Inbox<Chnl::InboxSpec>, Address<Chnl::AddressSpec>) -> Fut + Send + 'static,
    Fut: Future<Output = Chld::Output> + Send + 'static,
    Fut::Output: Send + 'static,
    Chld: ChildSpec,
    Chnl: ChannelSpec,
    Chnl::AddressSpec: Clone + Send + 'static,
    Chnl::InboxSpec: Send + 'static,
{
    spawn::<Chld, Chnl, Fun, Fut>(None, None, f)
}
