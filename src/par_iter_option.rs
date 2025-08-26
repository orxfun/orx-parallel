use crate::computations::{map_count, reduce_sum, reduce_unit};
use crate::{
    ChunkSize, DefaultRunner, IterationOrder, NumThreads, ParCollectInto, ParallelRunner, Sum,
};
use core::cmp::Ordering;

pub trait ParIterOption<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Item;

    // params transformations

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, order: IterationOrder) -> Self;

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIterOption<Q, Item = Self::Item>;

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterOption<R, Item = Out>
    where
        Self: Sized,
        Map: Fn(Self::Item) -> Out + Sync + Clone,
        Out: Send;

    fn filter<Filter>(self, filter: Filter) -> impl ParIterOption<R, Item = Self::Item>
    where
        Self: Sized,
        Filter: Fn(&Self::Item) -> bool + Sync + Clone,
        Self::Item: Send;

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIterOption<R, Item = IOut::Item>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Self::Item) -> IOut + Sync + Clone;

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIterOption<R, Item = Out>
    where
        Self: Sized,
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync + Clone,
        Out: Send;

    fn inspect<Operation>(self, operation: Operation) -> impl ParIterOption<R, Item = Self::Item>
    where
        Self: Sized,
        Operation: Fn(&Self::Item) + Sync + Clone,
        Self::Item: Send;

    // collect

    fn collect_into<C>(self, output: C) -> Option<C>
    where
        C: ParCollectInto<Self::Item>;

    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<Self::Item>;

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync;

    fn all<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        let violates = |x: &Self::Item| !predicate(x);
        self.find(violates).map(|x| x.is_none())
    }

    fn any<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
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
        Operation: Fn(Self::Item) + Sync,
    {
        let map = |x| operation(x);
        self.map(map).reduce(reduce_unit).map(|_| ())
    }

    fn max(self) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    fn max_by<Compare>(self, compare: Compare) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn min(self) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    fn min_by<Compare>(self, compare: Compare) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
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
        Self::Item: Sum<Out>,
        Out: Send,
    {
        self.map(Self::Item::map)
            .reduce(Self::Item::reduce)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }

    // early exit

    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send;

    fn find<Predicate>(self, predicate: Predicate) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
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
    #[inline(always)]
    fn into_option(self) -> Option<T> {
        self.ok()
    }
}
