use crate::{computations::heap_sort_into, values::Values};
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_split_vec::PseudoDefault;

pub enum ValuesPush<E> {
    Done,
    StoppedByWhileCondition { idx: usize },
    StoppedByError { idx: usize, error: E },
}

pub enum ThreadCollect<V>
where
    V: Values,
{
    AllCollected {
        vec: Vec<(usize, V::Item)>,
    },
    StoppedByWhileCondition {
        vec: Vec<(usize, V::Item)>,
        stopped_idx: usize,
    },
    StoppedByError {
        error: V::Error,
    },
}

pub enum ParallelCollect<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    AllCollected { pinned_vec: P },
    StoppedByWhileCondition { pinned_vec: P, stopped_idx: usize },
    StoppedByError { error: V::Error },
}

impl<V, P> ParallelCollect<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    pub fn reduce(results: Vec<ThreadCollect<V>>, mut pinned_vec: P) -> Self {
        let mut vectors = Vec::with_capacity(results.len());
        let mut min_stopped_idx = None;

        for x in results {
            match x {
                ThreadCollect::AllCollected { vec } => vectors.push(vec),
                ThreadCollect::StoppedByWhileCondition { vec, stopped_idx } => {
                    min_stopped_idx = match min_stopped_idx {
                        Some(x) => Some(core::cmp::min(x, stopped_idx)),
                        None => Some(stopped_idx),
                    };
                    vectors.push(vec);
                }
                ThreadCollect::StoppedByError { error } => return Self::StoppedByError { error },
            }
        }

        heap_sort_into(vectors, min_stopped_idx, &mut pinned_vec);

        match min_stopped_idx {
            Some(stopped_idx) => Self::StoppedByWhileCondition {
                pinned_vec,
                stopped_idx,
            },
            None => Self::AllCollected { pinned_vec },
        }
    }
}

impl<V, P> ParallelCollect<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    pub fn to_collected(self) -> P {
        match self {
            Self::AllCollected { pinned_vec } => pinned_vec,
            Self::StoppedByWhileCondition {
                pinned_vec,
                stopped_idx: _,
            } => pinned_vec,
            Self::StoppedByError { error: _ } => PseudoDefault::pseudo_default(),
        }
    }
}
