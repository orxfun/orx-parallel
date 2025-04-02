use crate::ParIter;

pub struct GenericIterator<T, S, R, O>
where
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
    sequential: S,
    rayon: R,
    orx: O,
}

impl<T, S, R, O> GenericIterator<T, S, R, O>
where
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
    pub fn new(sequential: S, rayon: R, orx: O) -> Self {
        Self {
            sequential,
            rayon,
            orx,
        }
    }

    // computation transformations

    pub fn map<Out, Map>(
        self,
        map: Map,
    ) -> GenericIterator<
        Out,
        impl Iterator<Item = Out>,
        impl rayon::iter::ParallelIterator<Item = Out>,
        impl ParIter<Item = Out>,
    >
    where
        Out: Send + Sync,
        Map: Fn(T) -> Out + Send + Sync + Clone,
    {
        let sequential = self.sequential.map(map.clone());
        let rayon = self.rayon.map(map.clone());
        let orx = self.orx.map(map);
        GenericIterator {
            sequential,
            rayon,
            orx,
        }
    }

    fn filter<Filter>(
        self,
        filter: Filter,
    ) -> GenericIterator<
        T,
        impl Iterator<Item = T>,
        impl rayon::iter::ParallelIterator<Item = T>,
        impl ParIter<Item = T>,
    >
    where
        Filter: Fn(&T) -> bool + Send + Sync + Clone,
    {
        let sequential = self.sequential.filter(filter.clone());
        let rayon = self.rayon.filter(filter.clone());
        let orx = self.orx.filter(filter);
        GenericIterator {
            sequential,
            rayon,
            orx,
        }
    }
}
