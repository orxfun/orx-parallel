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

    pub fn flat_map<IOut, FlatMap>(
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

    pub fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> GenericIterator<
        Out,
        impl Iterator<Item = Out>,
        impl rayon::iter::ParallelIterator<Item = Out>,
        impl ParIter<Item = Out>,
    >
    where
        Out: Send + Sync,
        FilterMap: Fn(T) -> Option<Out> + Send + Sync + Clone,
    {
        let sequential = self.sequential.filter_map(filter_map.clone());
        let rayon = self.rayon.filter_map(filter_map.clone());
        let orx = self.orx.filter_map(filter_map);
        GenericIterator {
            sequential,
            rayon,
            orx,
        }
    }

    pub fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> GenericIterator<
        T,
        impl Iterator<Item = T>,
        impl rayon::iter::ParallelIterator<Item = T>,
        impl ParIter<Item = T>,
    >
    where
        Operation: Fn(&T) + Sync + Send + Clone,
    {
        let sequential = self.sequential.inspect(operation.clone());
        let rayon = self.rayon.inspect(operation.clone());
        let orx = self.orx.inspect(operation);
        GenericIterator {
            sequential,
            rayon,
            orx,
        }
    }

    // special item transformations

    pub fn flatten(
        self,
    ) -> GenericIterator<
        <T as IntoIterator>::Item,
        impl Iterator<Item = <T as IntoIterator>::Item>,
        impl rayon::iter::ParallelIterator<Item = <T as IntoIterator>::Item>,
        impl ParIter<Item = <T as IntoIterator>::Item>,
    >
    where
        T: IntoIterator + rayon::iter::IntoParallelIterator<Item = <T as IntoIterator>::Item>,
        <T as IntoIterator>::IntoIter: Send + Sync,
        <T as IntoIterator>::Item: Send + Sync,
        R: Send + Sync,
        Self: Send + Sync,
    {
        let sequential = self.sequential.flatten();
        let rayon = self.rayon.flatten();
        let orx = self.orx.flatten();
        GenericIterator {
            sequential,
            rayon,
            orx,
        }
    }
}

// special item transformations

impl<'a, T, S, R, O> GenericIterator<&'a T, S, R, O>
where
    &'a T: Send + Sync,
    S: Iterator<Item = &'a T>,
    R: rayon::iter::ParallelIterator<Item = &'a T>,
    O: ParIter<Item = &'a T>,
{
    pub fn copied(
        self,
    ) -> GenericIterator<
        T,
        impl Iterator<Item = T> + use<'a, T, S, R, O>,
        impl rayon::iter::ParallelIterator<Item = T> + use<'a, T, S, R, O>,
        impl ParIter<Item = T> + use<'a, T, S, R, O>,
    >
    where
        T: Copy + Send + Sync + 'a,
    {
        let sequential = self.sequential.copied();
        let rayon = self.rayon.copied();
        let orx = self.orx.copied();
        GenericIterator {
            sequential,
            rayon,
            orx,
        }
    }

    pub fn cloned(
        self,
    ) -> GenericIterator<
        T,
        impl Iterator<Item = T> + use<'a, T, S, R, O>,
        impl rayon::iter::ParallelIterator<Item = T> + use<'a, T, S, R, O>,
        impl ParIter<Item = T> + use<'a, T, S, R, O>,
    >
    where
        T: Clone + Send + Sync + 'a,
    {
        let sequential = self.sequential.cloned();
        let rayon = self.rayon.cloned();
        let orx = self.orx.cloned();
        GenericIterator {
            sequential,
            rayon,
            orx,
        }
    }
}
