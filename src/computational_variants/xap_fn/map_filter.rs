use crate::computational_variants::xap_fn::{filter::Filter, map::Map};

pub trait MapFilter<I, O> {
    fn map_filter(&self, i: I) -> Option<O>;

    type Compose<Q, M3, F3>: MapFilter<I, Q>
    where
        M3: Map<O, Q>,
        F3: Filter<Q>;
    fn compose<Q, M3, F3>(self, m: M3, f: F3) -> Self::Compose<Q, M3, F3>
    where
        M3: Map<O, Q>,
        F3: Filter<Q>;

    type ComposeF<F3>: MapFilter<I, O>
    where
        F3: Fn(&O) -> bool;
    fn compose_f<F3>(self, f: F3) -> Self::ComposeF<F3>
    where
        F3: Fn(&O) -> bool;
}
