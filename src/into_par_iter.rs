use crate::ParIter;
use orx_concurrent_iter::IntoConcurrentIter;

pub trait IntoParIter {
    type Item: Send + Sync;

    fn into_par(self) -> impl ParIter<Item = Self::Item>;
}

// impl<I: IntoConcurrentIter> IntoParIter for I {
//     type Item = I::Item;

//     fn into_par(self) -> impl ParIter<Item = Self::Item> {
//         todo!()
//     }
// }
