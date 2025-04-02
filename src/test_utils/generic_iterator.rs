use crate::ParIter;

pub struct GenericIterator<T, S, R, O>
where
    T: Send + Sync,
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
    T: Send + Sync,
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

    pub fn filter<Filter>(
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

    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> GenericIterator<
        <IOut as IntoIterator>::Item,
        impl Iterator<Item = <IOut as IntoIterator>::Item>,
        impl rayon::iter::ParallelIterator<Item = <IOut as IntoIterator>::Item>,
        impl ParIter<Item = <IOut as IntoIterator>::Item>,
    >
    where
        IOut: IntoIterator
            + Send
            + Sync
            + rayon::iter::IntoParallelIterator<Item = <IOut as IntoIterator>::Item>,
        <IOut as IntoIterator>::IntoIter: Send + Sync,
        <IOut as IntoIterator>::Item: Send + Sync,
        FlatMap: Fn(T) -> IOut + Send + Sync + Clone,
    {
        let sequential = self.sequential.flat_map(flat_map.clone());
        let rayon = self.rayon.flat_map(flat_map.clone());
        let orx = self.orx.flat_map(flat_map);
        GenericIterator {
            sequential,
            rayon,
            orx,
        }
    }
}
