use crate::xap::{
    might_stop_iterators::MightStopMap,
    xap_trait::{IterOf, Xap},
};
use core::marker::PhantomData;

pub trait Stopper {
    type Elem<T>;

    // transformations

    type Map<X, Q, G>: IntoIterator<Item = Self::Elem<Q>>
    where
        X: Xap<S = Self>,
        G: Fn(X::O) -> Q;
}

pub enum NeverStop {}
impl Stopper for NeverStop {
    type Elem<T> = T;

    // transformations

    type Map<X, Q, G>
        = core::iter::Map<IterOf<X>, G>
    where
        X: Xap<S = Self>,
        G: Fn(X::O) -> Q;
}

pub struct MightStop<E>(PhantomData<E>);
impl<E> Stopper for MightStop<E> {
    type Elem<T> = Result<T, StoppedBy<E>>;

    // transformations

    type Map<X, Q, G>
        = MightStopMap<E, X, Q, G>
    where
        X: Xap<S = Self>,
        G: Fn(X::O) -> Q;
}

pub enum StoppedBy<E> {
    ByWhilst,
    ByError(E),
}
