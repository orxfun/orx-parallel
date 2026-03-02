use crate::fun_composition::map::map_trait::Map;

pub trait MapQ: Map {
    type Compose<X>: MapQ<I = Self::I, O = X::O>
    where
        X: Map<I = Self::O>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: Map<I = Self::O>;
}
