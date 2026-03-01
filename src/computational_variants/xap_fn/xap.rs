use crate::generic_values::{TransformableValues, Values};

pub trait XapFn<I, Vo>
where
    Vo: TransformableValues,
{
    fn run(&self, i: I) -> Vo;

    type Map<M, O>: XapFn<I, Vo::Map<M, O>>
    where
        M: Fn(<Vo as Values>::Item) -> O;
    fn map<M, O>(self, map: M) -> Self::Map<M, O>
    where
        M: Fn(<Vo as Values>::Item) -> O;

    type Filter<F>: XapFn<I, Vo::Filter<F>>
    where
        F: Fn(&<Vo as Values>::Item) -> bool;
    fn filter<F>(self, filter: F) -> Self::Filter<F>
    where
        F: Fn(&<Vo as Values>::Item) -> bool;
}
