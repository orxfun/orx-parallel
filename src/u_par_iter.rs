use crate::{DefaultRunner, ParIter, ParallelRunner, computations::Using};

/// Parallel iterator.
pub trait ParIterUsing<U, R = DefaultRunner>: ParIter<R>
where
    R: ParallelRunner,
    U: Using,
{
    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(&mut U::Item, Self::Item) -> Out + Send + Sync + Clone;
}
