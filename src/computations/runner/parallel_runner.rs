use super::thread_runner::ThreadRunner;
use crate::parameters::Params;

pub trait ParallelRunner: Sized {
    type ThreadRunner: ThreadRunner;

    fn new(params: Params) -> Self;
}
