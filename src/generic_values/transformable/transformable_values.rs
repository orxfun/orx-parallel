use crate::generic_values::{Values, runner_results::Fallible};

pub trait TransformableValues: Values {
    type Map<M, O>: TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(Self::Item) -> O;
    fn map<M, O>(self, map: M) -> Self::Map<M, O>
    where
        M: Fn(Self::Item) -> O;

    type Inspect<F>: TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        F: Fn(&Self::Item);
    fn inspect<F, O>(self, inspect: F) -> Self::Inspect<F>
    where
        F: Fn(&Self::Item);

    type Filter<F>: TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        F: Fn(&Self::Item) -> bool;
    fn filter<F>(self, filter: F) -> Self::Filter<F>
    where
        F: Fn(&Self::Item) -> bool;

    type FlatMap<Fm, Vo>: TransformableValues<Item = Vo::Item, Fallibility = Self::Fallibility>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMap<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    type FilterMap<Fm, O>: TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        Fm: Fn(Self::Item) -> Option<O>;
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMap<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>;

    type Whilst<W>: TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        W: Fn(&Self::Item) -> bool;
    fn whilst<W>(self, whilst: W) -> Self::Whilst<W>
    where
        W: Fn(&Self::Item) -> bool;

    type MapWhileOk<Mr, O, E>: Values<Item = O, Fallibility = Fallible<E>>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send;
    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> Self::MapWhileOk<Mr, O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send;

    type UMap<U, M, O>: TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(*mut U, Self::Item) -> O;
    fn u_map<U, M, O>(self, u: *mut U, map: M) -> Self::UMap<U, M, O>
    where
        M: Fn(*mut U, Self::Item) -> O;

    type UFilter<U, F>: TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        F: Fn(*mut U, &Self::Item) -> bool;
    fn u_filter<U, F>(self, u: *mut U, filter: F) -> Self::UFilter<U, F>
    where
        F: Fn(*mut U, &Self::Item) -> bool;

    type UFlatMap<U, Fm, Vo>: TransformableValues<Item = Vo::Item, Fallibility = Self::Fallibility>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo;
    fn u_flat_map<U, Fm, Vo>(self, u: *mut U, flat_map: Fm) -> Self::UFlatMap<U, Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo;

    type UFilterMap<U, Fm, O>: TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>;
    fn u_filter_map<U, Fm, O>(self, u: *mut U, filter_map: Fm) -> Self::UFilterMap<U, Fm, O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>;
}
