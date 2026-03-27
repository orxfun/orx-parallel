use crate::computation::fun::map::Map;

pub trait MapQueue: Map {
    type Then<Q, H>: MapQueue<I = Self::I, O = Q>
    where
        H: Map<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: Map<I = Self::O, O = Q>;
}
