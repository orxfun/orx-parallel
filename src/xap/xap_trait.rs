use crate::xap::stopper::Stopper;

pub type Elem<X> = Result<<X as Xap>::O, <X as Xap>::S>;

pub trait Xap {
    type I;

    type O;

    type S: Stopper;

    type Values: IntoIterator<Item = Elem<Self>>;

    fn xap(&self, i: Self::I) -> Self::Values;

    // transformations

    type Map<Q, G>: Xap<S = Self::S, I = Self::I, O = Q>
    where
        G: Fn(Self::O) -> Q;

    type Inspect<G>: Xap<S = Self::S, I = Self::I, O = Self::O>
    where
        G: Fn(&Self::O);

    type Filter<G>: Xap<S = Self::S, I = Self::I, O = Self::O>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>: Xap<S = Self::S, I = Self::I, O = Q>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>: Xap<S = Self::S, I = Self::I, O = V::Item>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;
}
