use crate::xap::might_stop_iterators::MightStopMap;
use crate::xap::xap_trait::{IterOf, Xap};
use core::marker::PhantomData;

pub trait Stopper {
    type Elem<T>;

    // transformations

    type Map<I, Q, G>: IntoIterator<Item = Self::Elem<Q>>
    where
        I: IntoIterator<Item = Self::Elem<X>>,
        G: Fn(I::Item) -> Q;

    fn map<I, Q, G>(iter: I, f: G) -> Self::Map<I, Q, G>
    where
        I: IntoIterator<Item = Self::Elem<X>>,
        G: Fn(I::Item) -> Q;
}

pub enum NeverStop {}
impl Stopper for NeverStop {
    type Elem<T> = T;

    // transformations

    type Map<I, Q, G>
        = core::iter::Map<I::IntoIter, G>
    where
        I: IntoIterator<Item = Self::Elem<X>>,
        G: Fn(I::Item) -> Q;

    fn map<I, Q, G>(iter: I, f: G) -> Self::Map<I, Q, G>
    where
        I: IntoIterator<Item = Self::Elem<X>>,
        G: Fn(I::Item) -> Q,
    {
        iter.into_iter().map(f)
    }
}

pub struct MightStop<E>(PhantomData<E>);
impl<E> Stopper for MightStop<E> {
    type Elem<T> = MightStopItem<T, E>;

    // transformations

    type Map<I, Q, G>
        = MightStopMap<E, X, Q, G>
    where
        I: Iterator<Item = MightStopItem<T, E>>,
        G: Fn(I::Item) -> Q;

    fn map<I, Q, G>(iter: I, f: G) -> Self::Map<I, Q, G>
    where
        I: IntoIterator<Item = Self::Elem<X>>,
        G: Fn(I::Item) -> Q,
    {
        MightStopMap::new(iter, f)
    }

    // type Map<X, Q, G>
    //     = MightStopMap<E, X, Q, G>
    // where
    //     X: Xap<S = Self>,
    //     G: Fn(X::O) -> Q;

    // fn map<X, Q, G>(iter: IterOf<X>, f: G) -> Self::Map<X, Q, G>
    // where
    //     X: Xap<S = Self>,
    //     G: Fn(X::O) -> Q,
    // {
    //     MightStopMap::new(iter, f)
    // }
}

pub enum StoppedBy<E> {
    ByWhilst,
    ByError(E),
}

pub type MightStopItem<T, E> = Result<T, StoppedBy<E>>;
