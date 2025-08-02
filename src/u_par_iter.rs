use crate::{
    DefaultRunner, ParIter, ParallelRunner,
    computations::{Using, UsingClone, reduce_unit},
};

/// Parallel iterator.
pub trait ParIterUsing<U, R = DefaultRunner>: ParIter<R>
where
    R: ParallelRunner,
    U: Using,
{
    // computation transformations

    fn map_u<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(&mut U::Item, Self::Item) -> Out + Send + Sync + Clone;

    fn filter_u<Filter>(self, filter: Filter) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Send + Sync + Clone;

    fn flat_map_u<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<U, R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Send + Sync + Clone;

    fn filter_map_u<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<U, R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Send + Sync + Clone;

    fn inspect_u<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Operation: Fn(&mut U::Item, &Self::Item) + Sync + Send + Clone,
    {
        let map = move |u: &mut U::Item, x: Self::Item| {
            operation(u, &x);
            x
        };
        self.map_u(map)
    }

    // reduce

    fn for_each_u<Operation>(self, operation: Operation)
    where
        Operation: Fn(&mut U::Item, Self::Item) + Sync + Send,
    {
        let map = |u: &mut U::Item, x: Self::Item| operation(u, x);
        let _ = self.map_u(map).reduce(reduce_unit);
    }
}

// into using

pub trait IntoParIterUsing<R>: ParIter<R>
where
    R: ParallelRunner,
{
    fn using<U>(
        self,
        using: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + Send;
}
