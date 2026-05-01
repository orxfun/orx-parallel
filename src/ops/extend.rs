use crate::{Par, ParCollectInto};

pub trait ExtendPar<T>: ParCollectInto<T> {
    fn extend_par(&mut self, iter: impl Par<Item = T>) {}
}
