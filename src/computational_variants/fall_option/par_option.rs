use crate::computational_variants::Par;
use crate::default_fns::{map_count, map_self, reduce_sum, reduce_unit};
use crate::executor::parallel_compute as prc;
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, Sum};
use core::cmp::Ordering;
use core::marker::PhantomData;
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
pub struct ParOption<I, T, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Option<T>>,
{
    par: Par<I, R>,
    phantom: PhantomData<T>,
}

impl<I, T, R> ParOption<I, T, R>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Option<T>>,
{
    pub(crate) fn new(par: Par<I, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }

    pub fn con_iter_len(&self) -> Option<usize> {
        self.par.con_iter().try_get_len()
    }

    pub fn into_regular_par(self) -> Par<I, R> {
        self.par
    }

    pub fn from_regular_par(regular_par: Par<I, R>) -> Self {
        Self {
            par: regular_par,
            phantom: PhantomData,
        }
    }

    // params transformations

    pub fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self {
        Self::new(self.par.num_threads(num_threads))
    }

    pub fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self {
        Self::new(self.par.chunk_size(chunk_size))
    }

    pub fn iteration_order(self, order: IterationOrder) -> Self {
        Self::new(self.par.iteration_order(order))
    }

    pub fn with_runner<Q: ParallelRunner>(self, orchestrator: Q) -> ParOption<I, T, Q> {
        let (_, params, iter) = self.par.destruct();
        ParOption::new(Par::new(orchestrator, params, iter))
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> char
    where
        Map: Fn(T) -> Out + Sync + Clone,
        Out: Send,
    {
        todo!()
    }

    fn filter<Filter>(self, filter: Filter) -> char
    where
        Self: Sized,
        Filter: Fn(&T) -> bool + Sync + Clone,
        T: Send,
    {
        todo!()
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> char
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(T) -> IOut + Sync + Clone,
    {
        todo!()
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> char
    where
        Self: Sized,
        FilterMap: Fn(T) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        todo!()
    }

    fn inspect<Operation>(self, operation: Operation) -> char
    where
        Self: Sized,
        Operation: Fn(&T) + Sync + Clone,
        T: Send,
    {
        todo!()
    }

    // collect

    pub fn collect_into<C>(self, output: C) -> Option<C>
    where
        T: Send,
        C: ParCollectInto<T>,
    {
        let (orchestrator, params, iter) = self.par.destruct();
        let x1 = |i: I::Item| match i {
            Some(x) => Ok(x),
            None => Err(()),
        };
        match output.x_try_collect_into(orchestrator, params, iter, x1) {
            Ok(x) => Some(x),
            Err(()) => None,
        }
    }

    fn collect<C>(self) -> Option<C>
    where
        T: Send,
        C: ParCollectInto<T>,
    {
        let output = C::empty(self.par.con_iter().try_get_len());
        self.collect_into(output)
    }

    // reduce

    pub fn reduce<Reduce>(self, reduce: Reduce) -> Option<Option<T>>
    where
        T: Send,
        Reduce: Fn(T, T) -> T + Sync,
    {
        let (orchestrator, params, iter) = self.par.destruct();
        let x1 = |i: I::Item| match i {
            Some(x) => Ok(x),
            None => Err(()),
        };
        match prc::reduce::x(orchestrator, params, iter, x1, reduce).1 {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    fn all<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        T: Send,
        Predicate: Fn(&T) -> bool + Sync,
    {
        let violates = |x: &T| !predicate(x);
        self.find(violates).map(|x| x.is_none())
    }

    fn any<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        T: Send,
        Predicate: Fn(&T) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    fn count(self) -> Option<usize> {
        let a = self.map(map_count);
        // self.map(map_count)
        //     .reduce(reduce_sum)
        //     .map(|x| x.unwrap_or(0))
        todo!()
    }

    fn for_each<Operation>(self, operation: Operation) -> Option<()>
    where
        Operation: Fn(T) + Sync,
    {
        let map = |x| operation(x);
        // self.map(map).reduce(reduce_unit).map(|_| ())
        todo!()
    }

    fn max(self) -> Option<Option<T>>
    where
        T: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    fn max_by<Compare>(self, compare: Compare) -> Option<Option<T>>
    where
        T: Send,
        Compare: Fn(&T, &T) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Option<T>>
    where
        Self: Sized,
        T: Send,
        Key: Ord,
        GetKey: Fn(&T) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn min(self) -> Option<Option<T>>
    where
        Self: Sized,
        T: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    fn min_by<Compare>(self, compare: Compare) -> Option<Option<T>>
    where
        Self: Sized,
        T: Send,
        Compare: Fn(&T, &T) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Option<T>>
    where
        Self: Sized,
        T: Send,
        Key: Ord,
        GetKey: Fn(&T) -> Key + Sync,
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
        T: Sum<Out>,
        Out: Send,
    {
        // self.map(T::map)
        //     .reduce(T::reduce)
        //     .map(|x| x.unwrap_or(T::zero()))
        todo!()
    }

    // early exit

    pub fn first(self) -> Option<Option<T>>
    where
        T: Send,
    {
        let (orchestrator, params, iter) = self.par.destruct();
        let x1 = |i: I::Item| match i {
            Some(x) => Ok(x),
            None => Err(()),
        };
        match params.iteration_order {
            IterationOrder::Ordered => {
                let (_, result) = prc::next::x(orchestrator, params, iter, x1);
                match result {
                    Ok(x) => Some(x.map(|y| y.1)),
                    Err(_) => None,
                }
            }
            IterationOrder::Arbitrary => {
                let (_, result) = prc::next_any::x(orchestrator, params, iter, x1);
                match result {
                    Ok(x) => Some(x),
                    Err(_) => None,
                }
            }
        }
    }

    fn find<Predicate>(self, predicate: Predicate) -> Option<Option<T>>
    where
        T: Send,
        Predicate: Fn(&T) -> bool + Sync,
    {
        //  self.filter(&predicate).first()
        None
    }
}
