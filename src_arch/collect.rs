use crate::Par;
use orx_concurrent_iter::ConcurrentIter;

pub trait FromPar {
    fn from_par<Data>(par: Par<Data>) -> Self
    where
        Data: ConcurrentIter;
}
