use crate::{DefaultRunner, ParCollectInto, ParallelRunner};

pub trait ParIterResult<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Item;

    type Error: Send;

    fn con_iter_len(&self) -> Option<usize>;

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>;

    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self: Sized,
    {
        let output = C::empty(self.con_iter_len());
        self.collect_into(output)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync;
}
