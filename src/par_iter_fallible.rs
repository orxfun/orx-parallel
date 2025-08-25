use crate::computations::{map_count, reduce_sum, reduce_unit};
use crate::{ChunkSize, IterationOrder, NumThreads, Sum};
use crate::{
    DefaultRunner, ParCollectInto, ParIter, ParallelRunner,
    values::fallible_iterators::ResultOfIter,
};
use core::cmp::Ordering;

pub trait ParIterFallible<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Success;

    type Error: Send;

    type RegularItem: IntoResult<Self::Success, Self::Error>;

    type RegularParIter: ParIter<R, Item = Self::RegularItem>;

    fn con_iter_len(&self) -> Option<usize>;

    fn into_regular_par(self) -> Self::RegularParIter;

    fn from_regular_par(regular_par: Self::RegularParIter) -> Self;

    // params transformations

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self
    where
        Self: Sized,
    {
        Self::from_regular_par(self.into_regular_par().num_threads(num_threads))
    }

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self
    where
        Self: Sized,
    {
        Self::from_regular_par(self.into_regular_par().chunk_size(chunk_size))
    }

    fn iteration_order(self, order: IterationOrder) -> Self
    where
        Self: Sized,
    {
        Self::from_regular_par(self.into_regular_par().iteration_order(order))
    }

    fn with_runner<Q: ParallelRunner>(
        self,
    ) -> impl ParIterFallible<Q, Success = Self::Success, Error = Self::Error>;

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterFallible<R, Success = Out, Error = Self::Error>
    where
        Self: Sized,
        Map: Fn(Self::Success) -> Out + Sync + Clone,
        Out: Send,
    {
        let par = self.into_regular_par();
        let map = par.map(move |x| x.into_result().map(map.clone()));
        map.into_fallible()
    }

    fn filter<Filter>(
        self,
        filter: Filter,
    ) -> impl ParIterFallible<R, Success = Self::Success, Error = Self::Error>
    where
        Self: Sized,
        Filter: Fn(&Self::Success) -> bool + Sync + Clone,
        Self::Success: Send,
    {
        let par = self.into_regular_par();
        let filter_map = par.filter_map(move |x| match x.into_result() {
            Ok(x) => match filter(&x) {
                true => Some(Ok(x)),
                false => None,
            },
            Err(e) => Some(Err(e)),
        });
        filter_map.into_fallible()
    }

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterFallible<R, Success = IOut::Item, Error = Self::Error>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Self::Success) -> IOut + Sync + Clone,
    {
        let par = self.into_regular_par();
        let map = par.flat_map(move |x| match x.into_result() {
            Ok(x) => ResultOfIter::ok(flat_map(x).into_iter()),
            Err(e) => ResultOfIter::err(e),
        });
        map.into_fallible()
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterFallible<R, Success = Out, Error = Self::Error>
    where
        Self: Sized,
        FilterMap: Fn(Self::Success) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        let par = self.into_regular_par();
        let filter_map = par.filter_map(move |x| match x.into_result() {
            Ok(x) => filter_map(x).map(|x| Ok(x)),
            Err(e) => Some(Err(e)),
        });
        filter_map.into_fallible()
    }

    fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterFallible<R, Success = Self::Success, Error = Self::Error>
    where
        Self: Sized,
        Operation: Fn(&Self::Success) + Sync + Clone,
        Self::Success: Send,
    {
        let map = move |x| {
            operation(&x);
            x
        };
        self.map(map)
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>;

    fn collect<C>(self) -> Result<C, Self::Error>
    where
        Self: Sized,
        C: ParCollectInto<Self::Success>,
    {
        let output = C::empty(self.con_iter_len());
        self.collect_into(output)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync;

    fn all<Predicate>(self, predicate: Predicate) -> Result<bool, Self::Error>
    where
        Self: Sized,
        Self::Success: Send,
        Predicate: Fn(&Self::Success) -> bool + Sync,
    {
        let violates = |x: &Self::Success| !predicate(x);
        self.find(violates).map(|x| x.is_none())
    }

    fn any<Predicate>(self, predicate: Predicate) -> Result<bool, Self::Error>
    where
        Self: Sized,
        Self::Success: Send,
        Predicate: Fn(&Self::Success) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.map(map_count)
            .reduce(reduce_sum)
            .map(|x| x.unwrap_or(0))
    }

    fn for_each<Operation>(self, operation: Operation) -> Result<(), Self::Error>
    where
        Self: Sized,
        Operation: Fn(Self::Success) + Sync,
    {
        let map = |x| operation(x);
        self.map(map).reduce(reduce_unit).map(|_| ())
    }

    fn max(self) -> Result<Option<Self::Success>, Self::Error>
    where
        Self: Sized,
        Self::Success: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    fn max_by<Compare>(self, compare: Compare) -> Result<Option<Self::Success>, Self::Error>
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

    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Result<Option<Self::Success>, Self::Error>
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

    fn min(self) -> Result<Option<Self::Success>, Self::Error>
    where
        Self: Sized,
        Self::Success: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    fn min_by<Compare>(self, compare: Compare) -> Result<Option<Self::Success>, Self::Error>
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

    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Result<Option<Self::Success>, Self::Error>
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

    fn sum<Out>(self) -> Result<Out, Self::Error>
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

    fn first(self) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send;

    fn find<Predicate>(self, predicate: Predicate) -> Result<Option<Self::Success>, Self::Error>
    where
        Self: Sized,
        Self::Success: Send,
        Predicate: Fn(&Self::Success) -> bool + Sync,
    {
        self.filter(&predicate).first()
    }
}

pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl<T, E> IntoResult<T, E> for Result<T, E> {
    #[inline(always)]
    fn into_result(self) -> Result<T, E> {
        self
    }
}
