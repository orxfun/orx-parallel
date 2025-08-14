use crate::{
    computations::heap_sort_into,
    values::{Values, runner_results::Fallability},
};
use core::fmt::Debug;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_split_vec::PseudoDefault;

pub enum OrderedPush<F: Fallability> {
    Done,
    StoppedByWhileCondition { idx: usize },
    StoppedByError { idx: usize, error: F::Error },
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
        error: <V::Fallability as Fallability>::Error,
    },
}

impl<V: Values> Debug for ThreadCollect<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllCollected { vec } => f
                .debug_struct("AllCollected")
                .field("vec-len", &vec.len())
                .finish(),
            Self::StoppedByWhileCondition { vec, stopped_idx } => f
                .debug_struct("StoppedByWhileCondition")
                .field("vec-len", &vec.len())
                .field("stopped_idx", stopped_idx)
                .finish(),
            Self::StoppedByError { error: _ } => f.debug_struct("StoppedByError").finish(),
        }
    }
}

impl<V: Values> ThreadCollect<V> {
    pub fn into_result(self) -> Result<Self, <V::Fallability as Fallability>::Error> {
        match self {
            Self::StoppedByError { error } => Err(error),
            _ => Ok(self),
        }
    }
}

pub enum ParallelCollect<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    AllCollected {
        pinned_vec: P,
    },
    StoppedByWhileCondition {
        pinned_vec: P,
        stopped_idx: usize,
    },
    StoppedByError {
        error: <V::Fallability as Fallability>::Error,
    },
}

impl<V, P> core::fmt::Debug for ParallelCollect<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllCollected { pinned_vec } => f
                .debug_struct("AllCollected")
                .field("pinned_vec.len()", &pinned_vec.len())
                .finish(),
            Self::StoppedByWhileCondition {
                pinned_vec,
                stopped_idx,
            } => f
                .debug_struct("StoppedByWhileCondition")
                .field("pinned_vec.len()", &pinned_vec.len())
                .field("stopped_idx", stopped_idx)
                .finish(),
            Self::StoppedByError { error: _ } => f.debug_struct("StoppedByError").finish(),
        }
    }
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

    pub fn to_collected(self) -> P {
        match self {
            Self::AllCollected { pinned_vec } => pinned_vec,
            Self::StoppedByWhileCondition {
                pinned_vec,
                stopped_idx: _,
            } => pinned_vec,
            Self::StoppedByError { error: _ } => PseudoDefault::pseudo_default(),
            // TODO: we should not be needing PseudoDefault; this will be called only when infallible
        }
    }

    pub fn to_result(self) -> Result<P, <V::Fallability as Fallability>::Error> {
        match self {
            Self::AllCollected { pinned_vec } => Ok(pinned_vec),
            Self::StoppedByWhileCondition {
                pinned_vec,
                stopped_idx: _,
            } => Ok(pinned_vec),
            Self::StoppedByError { error } => Err(error),
        }
    }
}
