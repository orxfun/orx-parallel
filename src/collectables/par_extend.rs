use crate::{Par, collectables::ParExtendCore};

pub trait ParExtend<T>: ParExtendCore<T> {
    fn par_extend(&mut self, iter: impl Par<Item = T>)
    where
        T: Send;
}

impl<T, P: ParExtendCore<T>> ParExtend<T> for P {
    fn par_extend(&mut self, iter: impl Par<Item = T>)
    where
        T: Send,
    {
        iter.collect_into(self);
    }
}
