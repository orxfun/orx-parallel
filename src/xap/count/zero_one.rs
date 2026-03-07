use crate::xap::count::{Count, Many};
use crate::xap::fun::filter::FilterFn;
use crate::xap::fun::map::MapFn;

pub struct ZeroOne;

impl Count for ZeroOne {
    // transformations

    type ThenZeroOne = ZeroOne;

    type ThenOne = ZeroOne;

    type ThenMany = Many;

    // map

    type Map<I: IntoIterator, G: MapFn<I = I::Item>> = Option<G::O>;

    #[inline(always)]
    fn map<I: IntoIterator, G: MapFn<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        i.into_iter().next().map(|x| g.map(x))
    }

    // filter

    type Filter<I: IntoIterator, G: FilterFn<I = I::Item>> = Option<I::Item>;

    #[inline(always)]
    fn filter<I: IntoIterator, G: FilterFn<I = I::Item>>(i: I, g: G) -> Self::Filter<I, G> {
        match i.into_iter().next() {
            Some(x) if g.filter(&x) => Some(x),
            _ => None,
        }
    }
}
