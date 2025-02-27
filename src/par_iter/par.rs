use crate::collect_into::ParCollectInto;

pub trait Par: Sized {
    type Item: Send + Sync;

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        todo!()
    }
}
