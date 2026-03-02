use crate::fun_composition::flat_map::flat_map_trait::FlatMap;

pub trait FlatMapQ: FlatMap {
    type Compose<X>: FlatMapQ<I = Self::I, O = X::O>
    where
        X: FlatMap<I = Self::O>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: FlatMap<I = Self::O>;
}
