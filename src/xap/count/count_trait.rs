use crate::xap::fun::map::MapFn;

pub trait Count {
    type ThenZeroOne: Count;

    type ThenOne: Count;

    type ThenMany: Count;

    // transformations

    type Map<I: IntoIterator, G: MapFn<I = I::Item>>: IntoIterator<Item = G::O>;

    fn map<I: IntoIterator, G: MapFn<I = I::Item>>(i: I, g: G) -> Self::Map<I, G>;
}
