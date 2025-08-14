use crate::{computations::heap_sort_into, values::Values};
use core::fmt::Debug;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_split_vec::PseudoDefault;

pub enum SequentialPush<E> {
    Done,
    StoppedByWhileCondition,
    StoppedByError { error: E },
}
