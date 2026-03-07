use crate::xap::fun::map::MapQ;

pub trait Count {
    type ThenZeroOne: Count;

    type ThenOne: Count;

    type ThenMany: Count;

    // transformations

    type Map<I: IntoIterator, G: MapQ<I = I::Item>>;

    fn map<I: IntoIterator, G: MapQ<I = I::Item>>(i: I, g: G) -> Self::Map<I, G>;
}
