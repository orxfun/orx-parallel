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

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Send + Sync + Clone;

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Send + Sync + Clone;

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Send + Sync + Clone;
}
