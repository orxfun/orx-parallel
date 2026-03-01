pub trait MapFilter<I, O> {
    fn map_filter(&self, i: I) -> Option<O>;

    type Compose<Y, Q>: MapFilter<I, Q>
    where
        Y: MapFilter<O, Q>;
    fn compose<Y, Q>(self, y: Y) -> Self::Compose<Y, Q>
    where
        Y: MapFilter<O, Q>;
}
