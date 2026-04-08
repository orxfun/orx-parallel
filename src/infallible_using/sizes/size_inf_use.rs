use crate::infallible_using::Xap;
use crate::infallible_using::fun::*;
use crate::sizes::Many;
use crate::sizes::Size;

pub trait SizeInf: Size {
    // transformations

    type Map<X, Q, H>: Xap<U = X::U, I = X::I, O = Q, Size = Self>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    type Inspect<X, H>: Xap<U = X::U, I = X::I, O = X::O, Size = Self>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    type Filter<X, H>: Xap<U = X::U, I = X::I, O = X::O, Size = Self::ThenBin>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    type FilterMap<X, Q, H>: Xap<U = X::U, I = X::I, O = Q, Size = Self::ThenBin>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    type FlatMap<X, V, H>: Xap<U = X::U, I = X::I, O = V::Item, Size = Many>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    fn flat_map<X, V, H>(x: X, h: H) -> Self::FlatMap<X, V, H>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    // transformations - helper

    type Mapped<X, M>: Xap<U = X::U, I = X::I, O = M::O, Size = Self>
    where
        X: Xap<Size = Self>,
        M: Map<U = X::U, I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<U = X::U, I = X::O>;
}
