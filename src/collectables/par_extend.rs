use crate::{Par, ParExtendCore};

pub trait ParExtend<T>: ParExtendCore<T> {
    fn par_extend(&mut self, iter: impl Par<Item = T>)
    where
        T: Send,
        Self: Sized,
    {
        iter.collect_into(self);
    }
}
