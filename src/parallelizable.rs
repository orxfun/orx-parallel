use crate::{computational_variants::Par, parameters::Params, runner::DefaultRunner};
use orx_concurrent_iter::ConcurrentIterable;

pub trait Parallelizable: ConcurrentIterable {
    fn par(&self) -> Par<<Self as ConcurrentIterable>::Iter, DefaultRunner> {
        Par::new(Params::default(), self.con_iter())
    }
}

impl<I> Parallelizable for I where I: ConcurrentIterable {}
