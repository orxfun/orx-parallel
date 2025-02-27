use crate::{
    collect_into::ParCollectInto,
    computations::{DefaultRunner, ParallelRunner},
};

pub trait ParCore {
    fn input_len(&self) -> Option<usize>;
}

pub trait Par<R = DefaultRunner>: ParCore + Sized
where
    R: ParallelRunner,
{
    type Item: Send + Sync;

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>;

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let output = C::empty(self.input_len());
        self.collect_into(output)
    }
}
