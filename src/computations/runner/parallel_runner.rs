use orx_concurrent_iter::{ConcurrentIter, Element, Enumeration};

use super::thread_runner::ThreadRunner;
use crate::parameters::Params;

pub trait ParallelRunner: Sized {
    type SharedState;

    type ThreadRunner: ThreadRunner;

    fn new(params: Params) -> Self;

    fn shared_state(&self) -> &Self::SharedState;

    fn run<E, I, T>(self, iter: &I, transform: &T)
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
        T: Fn(<E::Element as Element>::ElemOf<I::Item>),
    {
        let shared_state = self.shared_state();
    }
}
