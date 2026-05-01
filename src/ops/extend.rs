use crate::{InfalliblePar, ParCollectInto};

pub trait ExtendPar<T>: ParCollectInto<T> {
    fn extend_par(&mut self, iter: impl InfalliblePar<InfItem = T>)
    where
        T: Send,
    {
        iter.inf_collect_into(self);
    }
}
