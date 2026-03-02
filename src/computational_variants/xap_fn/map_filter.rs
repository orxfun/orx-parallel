pub trait MapFilter<I, O> {
    fn map_filter(&self, i: I) -> Option<O>;

    // type Compose<X3,Y3,Q

    // type Compose<Z, Q>: MapFilter<I, Q>
    // where
    //     Z: MapFilter<O, Q>;
    // fn compose<Z, Q>(self, z: Z) -> Self::Compose<Z, Q>
    // where
    //     Z: MapFilter<O, Q>;
}
