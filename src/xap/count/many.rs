use crate::xap::count::{Count, iter::MapIterMany};
use crate::xap::fun::map::MapFn;

pub struct Many;

impl Count for Many {
    type ThenZeroOne = Many;

    type ThenOne = Many;

    type ThenMany = Many;

    // transformations

    type Map<I: IntoIterator, G: MapFn<I = I::Item>> = MapIterMany<I::IntoIter, G>;

    #[inline(always)]
    fn map<I: IntoIterator, G: MapFn<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        MapIterMany::new(i.into_iter(), g)
    }
}
