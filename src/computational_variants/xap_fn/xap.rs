use crate::generic_values::{TransformableValues, Values};

pub trait XapFn<I, Vo>
where
    Vo: TransformableValues,
{
    fn run(&self, i: I) -> Vo;

    type Map<Y, Q>: XapFn<I, Vo::Map<Y, Q>>
    where
        Y: Fn(<Vo as Values>::Item) -> Q;
    fn map<Y, Q>(self, map: Y) -> Self::Map<Y, Q>
    where
        Y: Fn(<Vo as Values>::Item) -> Q;

    type Filter<Y>: XapFn<I, Vo::Filter<Y>>
    where
        Y: Fn(&<Vo as Values>::Item) -> bool;
    fn filter<Y>(self, filter: Y) -> Self::Filter<Y>
    where
        Y: Fn(&<Vo as Values>::Item) -> bool;
}
