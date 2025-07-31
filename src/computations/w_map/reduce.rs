use super::m::WithM;
use orx_concurrent_iter::ConcurrentIter;

impl<I, T, O, M1> WithM<I, T, O, M1>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
{
}
