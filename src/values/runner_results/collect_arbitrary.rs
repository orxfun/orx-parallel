use crate::values::Values;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_split_vec::PseudoDefault;

pub enum ArbitraryPush<E> {
    Done,
    StoppedByWhileCondition,
    StoppedByError { error: E },
}

pub enum ThreadCollectArbitrary<V>
where
    V: Values,
{
    AllCollected,
    StoppedByWhileCondition,
    StoppedByError { error: V::Error },
}

impl<V: Values> core::fmt::Debug for ThreadCollectArbitrary<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllCollected => write!(f, "AllCollected"),
            Self::StoppedByWhileCondition => write!(f, "StoppedByWhileCondition"),
            Self::StoppedByError { error: _ } => f.debug_struct("StoppedByError").finish(),
        }
    }
}

impl<V: Values> ThreadCollectArbitrary<V> {
    pub fn into_result(self) -> Result<Self, V::Error> {
        match self {
            Self::StoppedByError { error } => Err(error),
            _ => Ok(self),
        }
    }
}

pub enum ParallelCollectArbitrary<V, P>
where
    V: Values,
    P: IntoConcurrentPinnedVec<V::Item>,
{
    AllCollected { pinned_vec: P },
    StoppedByWhileCondition { pinned_vec: P },
    StoppedByError { error: V::Error },
}

impl<V, P> core::fmt::Debug for ParallelCollectArbitrary<V, P>
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
            Self::StoppedByWhileCondition { pinned_vec } => f
                .debug_struct("StoppedByWhileCondition")
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
    pub fn to_collected(self) -> P {
        match self {
            Self::AllCollected { pinned_vec } => pinned_vec,
            Self::StoppedByWhileCondition { pinned_vec } => pinned_vec,
            Self::StoppedByError { error: _ } => PseudoDefault::pseudo_default(),
            // TODO: we should not be needing PseudoDefault; this will be called only when infallible
        }
    }

    pub fn to_result(self) -> Result<P, V::Error> {
        match self {
            Self::AllCollected { pinned_vec } => Ok(pinned_vec),
            Self::StoppedByWhileCondition { pinned_vec } => Ok(pinned_vec),
            Self::StoppedByError { error } => Err(error),
        }
    }
}
