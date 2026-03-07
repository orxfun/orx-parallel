use crate::xap::count::{Count, Many};
use crate::xap::fun::map::MapFn;

pub struct ZeroOne;

impl Count for ZeroOne {
    type ThenZeroOne = ZeroOne;

    type ThenOne = ZeroOne;

    type ThenMany = Many;

    // transformations

    type Map<I: IntoIterator, G: MapFn<I = I::Item>> = Option<G::O>;

    #[inline(always)]
    fn map<I: IntoIterator, G: MapFn<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        i.into_iter().next().map(|x| g.map(x))
    }
}
