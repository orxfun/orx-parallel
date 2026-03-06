use crate::out::Out;

pub trait Xap {
    type I;

    type O: Out;

    // transformations

    type Map<Q, G>: Xap<I = Self::I, O = Q>
    where
        G: Fn(Self::O) -> Q;
}
