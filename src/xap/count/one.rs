use crate::xap::count::{Count, Many, ZeroOne};
use crate::xap::fun::filter::FilterFn;
use crate::xap::fun::map::MapFn;

pub struct One;

impl Count for One {
    // transformations

    type ThenZeroOne = ZeroOne;

    type ThenOne = One;

    type ThenMany = Many;

    // map

    type Map<I: IntoIterator, G: MapFn<I = I::Item>> = [G::O; 1];

    #[inline(always)]
    fn map<I: IntoIterator, G: MapFn<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        let x = unsafe { i.into_iter().next().unwrap_unchecked() };
        [g.map(x)]
    }

    // filter

    type Filter<I: IntoIterator, G: FilterFn<I = I::Item>> = Option<I::Item>;

    #[inline(always)]
    fn filter<I: IntoIterator, G: FilterFn<I = I::Item>>(i: I, g: G) -> Self::Filter<I, G> {
        let x = unsafe { i.into_iter().next().unwrap_unchecked() };
        match g.filter(&x) {
            true => Some(x),
            false => None,
        }
    }
}
