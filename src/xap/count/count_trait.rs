use crate::xap::fun::{filter::FilterFn, filter_map::FilterMapFn, map::MapFn};

pub trait Count {
    // transformations

    type ThenZeroOne: Count;

    type ThenOne: Count;

    type ThenMany: Count;

    // map

    type Map<I: IntoIterator, G: MapFn<I = I::Item>>: IntoIterator<Item = G::O>;

    fn map<I: IntoIterator, G: MapFn<I = I::Item>>(i: I, g: G) -> Self::Map<I, G>;

    // filter

    type Filter<I: IntoIterator, G: FilterFn<I = I::Item>>: IntoIterator<Item = I::Item>;

    fn filter<I: IntoIterator, G: FilterFn<I = I::Item>>(i: I, g: G) -> Self::Filter<I, G>;

    // filter_map

    type FilterMap<I: IntoIterator, G: FilterMapFn<I = I::Item>>: IntoIterator<Item = G::O>;

    fn filter_map<I: IntoIterator, G: FilterMapFn<I = I::Item>>(
        i: I,
        g: G,
    ) -> Self::FilterMap<I, G>;
}
