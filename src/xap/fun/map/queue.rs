use crate::xap::fun::map::r#fn::MapFn;

pub trait MapQ: MapFn {
    type Then<Q, H>: MapQ<I = Self::I, O = Q>
    where
        H: MapFn<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: MapFn<I = Self::O, O = Q>;
}
