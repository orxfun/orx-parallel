use crate::{DefaultRunner, computational_variants::Par};
use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;

pub trait IntoParIterRec
where
    Self: IntoIterator,
    <Self as IntoIterator>::Item: Send,
{
    fn into_par_rec<E, I>(
        self,
        extend: E,
    ) -> Par<ConcurrentRecursiveIter<<Self as IntoIterator>::Item, E, I>, DefaultRunner>
    where
        I: IntoIterator<Item = <Self as IntoIterator>::Item>,
        I::IntoIter: ExactSizeIterator,
        E: Fn(&<Self as IntoIterator>::Item) -> I + Sync;
}
