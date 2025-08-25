use crate::computations::{map_count, reduce_sum, reduce_unit};
use crate::{
    ChunkSize, DefaultRunner, IterationOrder, NumThreads, ParCollectInto, ParIterFallible,
    ParallelRunner, Sum,
};
use core::cmp::Ordering;

pub trait ParIterOptional<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Success;

    // params transformations

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, order: IterationOrder) -> Self;

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIterOptional<Q, Success = Self::Success>;

    // TODO: with runner

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterOptional<R, Success = Out>
    where
        Self: Sized,
        Map: Fn(Self::Success) -> Out + Sync + Clone,
        Out: Send;

    fn filter<Filter>(self, filter: Filter) -> impl ParIterOptional<R, Success = Self::Success>
    where
        Self: Sized,
        Filter: Fn(&Self::Success) -> bool + Sync + Clone,
        Self::Success: Send;

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterOptional<R, Success = IOut::Item>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Self::Success) -> IOut + Sync + Clone;

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterOptional<R, Success = Out>
    where
        Self: Sized,
        FilterMap: Fn(Self::Success) -> Option<Out> + Sync + Clone,
        Out: Send;

    fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterOptional<R, Success = Self::Success>
    where
        Self: Sized,
        Operation: Fn(&Self::Success) + Sync + Clone,
        Self::Success: Send;

    // collect

    fn collect_into<C>(self, output: C) -> Option<C>
    where
        C: ParCollectInto<Self::Success>;

    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<Self::Success>;

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Option<Self::Success>>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync;

    fn all<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Success: Send,
        Predicate: Fn(&Self::Success) -> bool + Sync,
    {
        let violates = |x: &Self::Success| !predicate(x);
        self.find(violates).map(|x| x.is_none())
    }

    fn any<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Success: Send,
        Predicate: Fn(&Self::Success) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    fn count(self) -> Option<usize>
    where
        Self: Sized,
    {
        self.map(map_count)
            .reduce(reduce_sum)
            .map(|x| x.unwrap_or(0))
    }

    fn for_each<Operation>(self, operation: Operation) -> Option<()>
    where
        Self: Sized,
        Operation: Fn(Self::Success) + Sync,
    {
        let map = |x| operation(x);
        self.map(map).reduce(reduce_unit).map(|_| ())
    }

    fn max(self) -> Option<Option<Self::Success>>
    where
        Self: Sized,
        Self::Success: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    fn max_by<Compare>(self, compare: Compare) -> Option<Option<Self::Success>>
    where
        Self: Sized,
        Self::Success: Send,
        Compare: Fn(&Self::Success, &Self::Success) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Option<Self::Success>>
    where
        Self: Sized,
        Self::Success: Send,
        Key: Ord,
        GetKey: Fn(&Self::Success) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn min(self) -> Option<Option<Self::Success>>
    where
        Self: Sized,
        Self::Success: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    fn min_by<Compare>(self, compare: Compare) -> Option<Option<Self::Success>>
    where
        Self: Sized,
        Self::Success: Send,
        Compare: Fn(&Self::Success, &Self::Success) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Option<Self::Success>>
    where
        Self: Sized,
        Self::Success: Send,
        Key: Ord,
        GetKey: Fn(&Self::Success) -> Key + Sync,
    {
        let reduce = |x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn sum<Out>(self) -> Option<Out>
    where
        Self: Sized,
        Self::Success: Sum<Out>,
        Out: Send,
    {
        self.map(Self::Success::map)
            .reduce(Self::Success::reduce)
            .map(|x| x.unwrap_or(Self::Success::zero()))
    }

    // early exit

    fn first(self) -> Option<Option<Self::Success>>
    where
        Self::Success: Send;

    fn find<Predicate>(self, predicate: Predicate) -> Option<Option<Self::Success>>
    where
        Self: Sized,
        Self::Success: Send,
        Predicate: Fn(&Self::Success) -> bool + Sync,
    {
        self.filter(&predicate).first()
    }
}

pub trait IntoOption<T> {
    fn into_option(self) -> Option<T>;

    fn into_result_with_unit_err(self) -> Result<T, ()>;
}

impl<T> IntoOption<T> for Option<T> {
    #[inline(always)]
    fn into_option(self) -> Option<T> {
        self
    }

    #[inline(always)]
    fn into_result_with_unit_err(self) -> Result<T, ()> {
        match self {
            Some(x) => Ok(x),
            None => Err(()),
        }
    }
}

pub(crate) trait ResultIntoOption<T> {
    fn into_option(self) -> Option<T>;
}

impl<T> ResultIntoOption<T> for Result<T, ()> {
    fn into_option(self) -> Option<T> {
        match self {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }
}
