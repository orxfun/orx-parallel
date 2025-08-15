use super::x::X;
use crate::runner::parallel_runner_compute::reduce;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::values::Values;
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Sync,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, result) = reduce::x(R::reduce(p, len), self, reduce);
        let acc = match result {
            Ok(acc) => acc,
            Err(_) => None,
        };
        // let acc = unsafe { result.unwrap_unchecked() };
        (num_threads, acc)
    }
}
