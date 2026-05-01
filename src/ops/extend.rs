use crate::{ParCollectInto, ParInfCommon};

pub trait ParExtend<T>: ParCollectInto<T> {
    fn par_extend(&mut self, iter: impl ParInfCommon<InfItem = T>)
    where
        T: Send,
    {
        iter.inf_collect_into(self);
    }
}

impl<T, C: ParCollectInto<T>> ParExtend<T> for C {}
