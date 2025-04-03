use super::iter::GenericIterator;
use crate::ParIter;

impl<T, S, R, O> GenericIterator<T, S, R, O>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
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
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.map(map)),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.map(map)),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.map(map)),
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
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.filter(filter)),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.filter(filter)),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.filter(filter)),
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
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.flat_map(flat_map)),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.flat_map(flat_map)),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.flat_map(flat_map)),
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
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.filter_map(filter_map)),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.filter_map(filter_map)),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.filter_map(filter_map)),
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
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.inspect(operation)),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.inspect(operation)),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.inspect(operation)),
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
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.flatten()),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.flatten()),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.flatten()),
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
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.copied()),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.copied()),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.copied()),
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
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.cloned()),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.cloned()),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.cloned()),
        }
    }
}
