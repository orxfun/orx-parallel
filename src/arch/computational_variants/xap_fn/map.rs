pub trait Map<I, O> {
    fn map(&self, i: I) -> O;

    type Compose<Y, Q>: Map<I, Q>
    where
        Y: Fn(O) -> Q;
    fn compose<Y, Q>(self, y: Y) -> Self::Compose<Y, Q>
    where
        Y: Fn(O) -> Q;
}
