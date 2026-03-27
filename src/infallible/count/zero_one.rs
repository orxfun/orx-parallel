use crate::infallible::count::{Count, Many};
use crate::infallible::fun::{filter_map::FilterMap, flat_map::FlatMap, map::Map};

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

    // filter_map

    type FilterMap<I: IntoIterator, G: FilterMap<I = I::Item>> = Option<G::O>;

    #[inline(always)]
    fn filter_map<I: IntoIterator, G: FilterMap<I = I::Item>>(i: I, g: G) -> Self::FilterMap<I, G> {
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
