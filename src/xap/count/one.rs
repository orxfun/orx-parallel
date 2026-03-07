use crate::xap::count::{Count, Many, ZeroOne};
use crate::xap::fun::map::MapFn;

pub struct One;

impl Count for One {
    type ThenZeroOne = ZeroOne;

    type ThenOne = One;

    type ThenMany = Many;

    // transformations

    type Map<I: IntoIterator, G: MapFn<I = I::Item>> = [G::O; 1];

    #[inline(always)]
    fn map<I: IntoIterator, G: MapFn<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        let x = unsafe { i.into_iter().next().unwrap_unchecked() };
        [g.map(x)]
    }
}
