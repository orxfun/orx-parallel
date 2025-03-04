use crate::{
    runner::{DefaultRunner, ParallelRunner},
    ParIter,
};
use orx_concurrent_iter::IntoConcurrentIter;

pub trait IntoParIter<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Item: Send + Sync;

    fn into_par(self) -> impl ParIter<R, Item = Self::Item>;
}

// impl<I: IntoConcurrentIter> IntoParIter for I {
//     type Item = I::Item;

//     fn into_par(self) -> impl ParIter<Item = Self::Item> {
//         todo!()
//     }
// }
