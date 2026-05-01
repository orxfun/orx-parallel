use crate::ParCollectInto;
use crate::common_par_traits::{ParInfCommon, ParOptCommon, ParResCommon};

pub trait ParExtend<T>: ParCollectInto<T> {
    fn par_extend(&mut self, iter: impl ParInfCommon<CommonItem = T>)
    where
        T: Send,
    {
        iter.common_collect_into(self)
    }

    fn par_extend_opt(&mut self, iter: impl ParOptCommon<CommonItem = T>) -> Option<()>
    where
        T: Send,
    {
        iter.common_collect_into(self)
    }

    fn par_extend_res<E>(
        &mut self,
        iter: impl ParResCommon<CommonItem = T, CommonError = E>,
    ) -> Result<(), E>
    where
        T: Send,
        E: Send,
    {
        iter.common_collect_into(self)
    }
}

impl<T, C: ParCollectInto<T>> ParExtend<T> for C {}
