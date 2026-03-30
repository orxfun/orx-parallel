use crate::result::fun::map::fn_trait::MapRes;

pub trait MapResQueue: MapRes {
    type Then<Q, H>: MapResQueue<I = Self::I, O = Q>
    where
        H: MapRes<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: MapRes<I = Self::O, O = Q>;
}
