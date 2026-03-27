use crate::infallible::fun::flat_map::FlatMap;

pub trait FlatMapQueue: FlatMap {
    type Then<Q, H>: FlatMapQueue<I = Self::I>
    where
        Q: IntoIterator,
        H: FlatMap<I = <Self::O as IntoIterator>::Item, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        Q: IntoIterator,
        H: FlatMap<I = <Self::O as IntoIterator>::Item, O = Q>;
}
