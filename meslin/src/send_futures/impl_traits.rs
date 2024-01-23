// use std::fmt::Debug;

// use crate::*;

// impl<'a, S: IsSender, M> Debug for SendMsgWithFut<'a, S, M>
// where
//     S::With: Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("SendMsgWithFut")
//             .field("sender", &self.inner)
//             .field("msg", &self.with)
//             .finish()
//     }
// }
