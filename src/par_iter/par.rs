use crate::{
    collect_into::ParCollectInto,
    computations::{DefaultRunner, ParallelRunner},
};

pub trait Par<R = DefaultRunner>: Sized
where
    R: ParallelRunner,
{
    type Item: Send + Sync;

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        todo!()
    }
}
