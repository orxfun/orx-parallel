pub trait MapFilter<I, O> {
    fn map_filter(&self, i: I) -> Option<O>;

    type Compose<O3, X3, Y3>: MapFilter<I, O3>
    where
        X3: Fn(O) -> O3,
        Y3: Fn(&O3) -> bool;
    fn compose<O3, X3, Y3>(self, m: X3, f: Y3) -> Self::Compose<O3, X3, Y3>
    where
        X3: Fn(O) -> O3,
        Y3: Fn(&O3) -> bool;
}
