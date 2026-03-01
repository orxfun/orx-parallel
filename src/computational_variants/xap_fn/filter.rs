pub trait Filter<I> {
    fn filter(&self, i: &I) -> bool;

    type Compose<Y>: Filter<I>
    where
        Y: Fn(&I) -> bool;
    fn compose<Y: Fn(&I) -> bool>(self, y: Y) -> Self::Compose<Y>;
}
