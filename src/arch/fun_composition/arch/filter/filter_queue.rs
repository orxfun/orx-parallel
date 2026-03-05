use crate::fun_composition::filter::filter_trait::Filter;

pub trait FilterQ: Filter {
    type Compose<X>: FilterQ<I = Self::I>
    where
        X: Filter<I = Self::I>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: Filter<I = Self::I>;
}
