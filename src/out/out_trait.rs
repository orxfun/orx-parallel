use crate::out::stop::Stopper;

pub trait Out {
    type Elem;

    type Stopper: Stopper;

    type Values: IntoIterator<Item = Result<Self::Elem, Self::Stopper>>;

    fn values(self) -> Self::Values;

    // transformations

    // type Map<Q, G>: Xap<I = Self::I, O = Q>
    // where
    //     G: Fn(Self::O) -> Q;
}
