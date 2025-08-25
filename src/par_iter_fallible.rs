use crate::{DefaultRunner, ParCollectInto, ParIter, ParallelRunner};

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
        let filter_map = par.filter_map(move |x| {
            let x = x.into_result();
            match x {
                Ok(x) => match filter(&x) {
                    true => Some(Ok(x)),
                    false => None,
                },
                Err(e) => Some(Err(e)),
            }
        });
        filter_map.into_fallible()
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>;

    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>,
        Self: Sized,
    {
        let output = C::empty(self.con_iter_len());
        self.collect_into(output)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync;

    // early exit

    fn first(self) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send;

    // fn find<Predicate>(self, predicate: Predicate) -> Result<Option<Self::Success>, Self::Error>
    // where
    //     Self::Success: Send,
    //     Predicate: Fn(&Self::Success) -> bool + Sync,
    // {
    //     self.filter(&predicate).first()
    // }
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
