use crate::values::Values;

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
