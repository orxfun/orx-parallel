use crate::xap::count::{Count, iter::MapIterMany};
use crate::xap::fun::map::MapQ;

pub struct Many;

impl Count for Many {
    type ThenZeroOne = Many;

    type ThenOne = Many;

    type ThenMany = Many;

    // transformations

    type Map<I: IntoIterator, G: MapQ<I = I::Item>> = MapIterMany<I::IntoIter, G>;

    fn map<I: IntoIterator, G: MapQ<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
        MapIterMany::new(i.into_iter(), g)
    }
}
