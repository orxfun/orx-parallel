use crate::{DefaultRunner, ParCollectInto, ParIter, ParallelRunner};

pub trait ParIterResult<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Success;

    type Error: Send;

    fn con_iter_len(&self) -> Option<usize>;

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>;

    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>,
        Self: Sized,
    {
        let output = C::empty(self.con_iter_len());
        self.collect_into(output)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync;
}

pub trait IntoFallibleParIter: ParIter<Item = Result<Self::Success, Self::Error>> {
    type Success;

    type Error: Send;
}
