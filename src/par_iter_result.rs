use crate::{DefaultRunner, ParCollectInto, ParallelRunner};

pub trait ParIterResult<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Item;

    type Error;

    fn con_iter_len(&self) -> Option<usize>;

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Error: Send;

    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Error: Send,
        Self: Sized,
    {
        let output = C::empty(self.con_iter_len());
        self.collect_into(output)
    }
}
