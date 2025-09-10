use crate::ParallelRunner;
use std::marker::PhantomData;

pub struct StdOrchestrator<R>
where
    R: ParallelRunner,
{
    r: PhantomData<R>,
}
