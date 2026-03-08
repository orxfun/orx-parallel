use crate::xap::count::iter::{FilterIterMany, FilterMapIterMany, FlatMapIterMany};
use crate::xap::count::{Count, iter::MapIterMany};
use crate::xap::fun::filter::FilterFn;
use crate::xap::fun::filter_map::FilterMapFn;
use crate::xap::fun::flat_map::FlatMapFn;
use crate::xap::fun::map::MapFn;

pub struct Many;

impl Count for Many {
    // transformations

    type ThenZeroOne = Many;

    type ThenOne = Many;

    type ThenMany = Many;

    // map

    type Map<I: IntoIterator, G: MapFn<I = I::Item>> = MapIterMany<I::IntoIter, G>;

    #[inline(always)]
    fn map<I: IntoIterator, G: MapFn<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        MapIterMany::new(i.into_iter(), g)
    }

    // filter

    type Filter<I: IntoIterator, G: FilterFn<I = I::Item>> = FilterIterMany<I::IntoIter, G>;

    #[inline(always)]
    fn filter<I: IntoIterator, G: FilterFn<I = I::Item>>(i: I, g: G) -> Self::Filter<I, G> {
        FilterIterMany::new(i.into_iter(), g)
    }

    // filter_map

    type FilterMap<I: IntoIterator, G: FilterMapFn<I = I::Item>> =
        FilterMapIterMany<I::IntoIter, G>;

    #[inline(always)]
    fn filter_map<I: IntoIterator, G: FilterMapFn<I = I::Item>>(
        i: I,
        g: G,
    ) -> Self::FilterMap<I, G> {
        FilterMapIterMany::new(i.into_iter(), g)
    }

    // flat_map

    type FlatMap<I: IntoIterator, G: FlatMapFn<I = I::Item>> = FlatMapIterMany<I::IntoIter, G>;

    #[inline(always)]
    fn flat_map<I: IntoIterator, G: FlatMapFn<I = I::Item>>(i: I, g: G) -> Self::FlatMap<I, G> {
        FlatMapIterMany::new(i.into_iter(), g)
    }
}
