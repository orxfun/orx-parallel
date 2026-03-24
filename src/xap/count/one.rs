use crate::xap::count::{Count, Many, ZeroOne};
use crate::xap::fun::filter_map::FilterMap;
use crate::xap::fun::flat_map::FlatMap;
use crate::xap::fun::map::Map;

pub struct One;

impl Count for One {
    // transformations

    type ThenZeroOne = ZeroOne;

    type ThenOne = One;

    type ThenMany = Many;

    // map

    type Map<I: IntoIterator, G: Map<I = I::Item>> = [G::O; 1];

    #[inline(always)]
    fn map<I: IntoIterator, G: Map<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        let x = unsafe { i.into_iter().next().unwrap_unchecked() };
        [g.map(x)]
    }

    // filter_map

    type FilterMap<I: IntoIterator, G: FilterMap<I = I::Item>> = Option<G::O>;

    #[inline(always)]
    fn filter_map<I: IntoIterator, G: FilterMap<I = I::Item>>(i: I, g: G) -> Self::FilterMap<I, G> {
        let x = unsafe { i.into_iter().next().unwrap_unchecked() };
        g.filter_map(x)
    }

    // flat_map

    type FlatMap<I: IntoIterator, G: FlatMap<I = I::Item>> = G::O;

    #[inline(always)]
    fn flat_map<I: IntoIterator, G: FlatMap<I = I::Item>>(i: I, g: G) -> Self::FlatMap<I, G> {
        let x = unsafe { i.into_iter().next().unwrap_unchecked() };
        g.flat_map(x)
    }
}
