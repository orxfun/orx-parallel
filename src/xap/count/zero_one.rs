use crate::xap::count::{Count, Many};
use crate::xap::fun::filter::Filter;
use crate::xap::fun::filter_map::FilterMap;
use crate::xap::fun::flat_map::FlatMap;
use crate::xap::fun::map::Map;

pub struct ZeroOne;

impl Count for ZeroOne {
    // transformations

    type ThenZeroOne = ZeroOne;

    type ThenOne = ZeroOne;

    type ThenMany = Many;

    // map

    type Map<I: IntoIterator, G: Map<I = I::Item>> = Option<G::O>;

    #[inline(always)]
    fn map<I: IntoIterator, G: Map<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        i.into_iter().next().map(|x| g.map(x))
    }

    // filter

    type Filter<I: IntoIterator, G: Filter<I = I::Item>> = Option<I::Item>;

    #[inline(always)]
    fn filter<I: IntoIterator, G: Filter<I = I::Item>>(i: I, g: G) -> Self::Filter<I, G> {
        match i.into_iter().next() {
            Some(x) if g.filter(&x) => Some(x),
            _ => None,
        }
    }

    // filter_map

    type FilterMap<I: IntoIterator, G: FilterMap<I = I::Item>> = Option<G::O>;

    #[inline(always)]
    fn filter_map<I: IntoIterator, G: FilterMap<I = I::Item>>(
        i: I,
        g: G,
    ) -> Self::FilterMap<I, G> {
        match i.into_iter().next() {
            Some(x) => g.filter_map(x),
            _ => None,
        }
    }

    // flat_map

    type FlatMap<I: IntoIterator, G: FlatMap<I = I::Item>> =
        core::iter::Flatten<core::option::IntoIter<G::O>>;

    #[inline(always)]
    fn flat_map<I: IntoIterator, G: FlatMap<I = I::Item>>(i: I, g: G) -> Self::FlatMap<I, G> {
        let x = {
            match i.into_iter().next() {
                Some(x) => Some(g.flat_map(x)),
                _ => None,
            }
        };
        x.into_iter().flatten()
    }
}
