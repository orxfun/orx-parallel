pub type IterOf<'i, X> = <<X as Xap>::Values<'i> as IntoIterator>::IntoIter;

pub trait Xap {
    type I;

    type O;

    type Values<'i>: IntoIterator<Item = Self::O>
    where
        Self: 'i;

    fn xap(&self, i: Self::I) -> Self::Values<'_>;

    // transformations

    type Map<Q, G>: Xap<I = Self::I, O = Q>
    where
        G: Fn(Self::O) -> Q;

    type Inspect<G>: Xap<I = Self::I, O = Self::O>
    where
        G: Fn(&Self::O);

    type Filter<G>: Xap<I = Self::I, O = Self::O>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>: Xap<I = Self::I, O = Q>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>: Xap<I = Self::I, O = V::Item>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;
}
