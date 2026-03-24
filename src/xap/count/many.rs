use crate::xap::count::iter::{FilterMapIterMany, FlatMapIterMany};
use crate::xap::count::{Count, iter::MapIterMany};
use crate::xap::fun::filter_map::FilterMap;
use crate::xap::fun::flat_map::FlatMap;
use crate::xap::fun::map::Map;

pub struct Many;

impl Count for Many {
    // transformations

    type ThenZeroOne = Many;

    type ThenOne = Many;

    type ThenMany = Many;

    // map

    type Map<I: IntoIterator, G: Map<I = I::Item>> = MapIterMany<I::IntoIter, G>;

    #[inline(always)]
    fn map<I: IntoIterator, G: Map<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        MapIterMany::new(i.into_iter(), g)
    }

    // filter_map

    type FilterMap<I: IntoIterator, G: FilterMap<I = I::Item>> = FilterMapIterMany<I::IntoIter, G>;

    #[inline(always)]
    fn filter_map<I: IntoIterator, G: FilterMap<I = I::Item>>(i: I, g: G) -> Self::FilterMap<I, G> {
        FilterMapIterMany::new(i.into_iter(), g)
    }

    // flat_map

    type FlatMap<I: IntoIterator, G: FlatMap<I = I::Item>> = FlatMapIterMany<I::IntoIter, G>;

    #[inline(always)]
    fn flat_map<I: IntoIterator, G: FlatMap<I = I::Item>>(i: I, g: G) -> Self::FlatMap<I, G> {
        FlatMapIterMany::new(i.into_iter(), g)
    }
}
