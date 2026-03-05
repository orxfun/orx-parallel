use crate::fun_composition::filter_map::filter_map_trait::FilterMap;

pub trait FilterMapQ: FilterMap {
    type Compose<X>: FilterMapQ<I = Self::I, O = X::O>
    where
        X: FilterMap<I = Self::O>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: FilterMap<I = Self::O>;
}
