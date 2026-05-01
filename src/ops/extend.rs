use crate::ParCollectInto;
use crate::common_par_traits::ParInfCommon;

pub trait ParExtend<T>: ParCollectInto<T> {
    fn par_extend(&mut self, iter: impl ParInfCommon<CommonItem = T>)
    where
        T: Send,
    {
        iter.common_collect_into(self);
    }
}

impl<T, C: ParCollectInto<T>> ParExtend<T> for C {}
