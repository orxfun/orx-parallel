use crate::generic_values::{Values, runner_results::Fallibility};
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub enum ArbitraryPush<F: Fallibility> {
    Done,
    StoppedByWhileCondition,
    StoppedByError { error: F::Error },
}

pub enum ThreadCollectArbitrary<F>
where
    F: Fallibility,
{
    AllCollected,
    StoppedByWhileCondition,
    StoppedByError { error: F::Error },
}

impl<F: Fallibility> ThreadCollectArbitrary<F> {
    pub fn into_result(self) -> Result<(), F::Error> {
        match self {
            Self::StoppedByError { error } => Err(error),
            _ => Ok(()),
        }
    }
}

impl<F: Fallibility> core::fmt::Debug for ThreadCollectArbitrary<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AllCollected => write!(f, "AllCollected"),
            Self::StoppedByWhileCondition => write!(f, "StoppedByWhileCondition"),
            Self::StoppedByError { error: _ } => f.debug_struct("StoppedByError").finish(),
        }
    }
}

pub enum ParallelCollectArbitrary<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    AllOrUntilWhileCollected {
        pinned_vec: P,
    },

    StoppedByError {
        error: <V::Fallibility as Fallibility>::Error,
    },
}

impl<V, P> core::fmt::Debug for ParallelCollectArbitrary<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AllOrUntilWhileCollected { pinned_vec } => f
                .debug_struct("AllCollected")
                .field("pinned_vec.len()", &pinned_vec.len())
                .finish(),
            Self::StoppedByError { error: _ } => f.debug_struct("StoppedByError").finish(),
        }
    }
}

impl<V, P> ParallelCollectArbitrary<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    pub fn into_result(self) -> Result<P, <V::Fallibility as Fallibility>::Error> {
        match self {
            Self::AllOrUntilWhileCollected { pinned_vec } => Ok(pinned_vec),
            Self::StoppedByError { error } => Err(error),
        }
    }
}
