pub type IterOf<'i, X> = <<X as Xap>::Values<'i> as IntoIterator>::IntoIter;

pub trait Xap {
    type I;

    type O;

    type Values<'i>: IntoIterator<Item = Self::O>
    where
        Self: 'i;

    fn xap(&self, i: Self::I) -> Self::Values<'_>;

    // transformations

    type Map<Q, H>: Xap<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Q;

    type Inspect<H>: Xap<I = Self::I, O = Self::O>
    where
        H: Fn(&Self::O);

    type Filter<H>: Xap<I = Self::I, O = Self::O>
    where
        H: Fn(&Self::O) -> bool;

    type FilterMap<Q, H>: Xap<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, H>: Xap<I = Self::I, O = V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;
}
