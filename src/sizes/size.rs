use crate::infallible::Xap;
use crate::infallible::fun::Map;
use crate::sizes::{Many, One, SizePair};

pub trait Size: Clone + Copy + Send + Default {
    type ThenBin: Size;

    type IntoPair: SizePair<S1 = Self, S2 = One>;

    fn elem_len() -> Option<usize>;

    fn output_len(input_len: Option<usize>) -> Option<usize> {
        match (input_len, Self::elem_len()) {
            (Some(input_len), Some(elem_len)) => Some(input_len * elem_len),
            _ => None,
        }
    }

    // transformations

    type Map<X, Q, H>: Xap<I = X::I, O = Q, Size = Self>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    type Inspect<X, H>: Xap<I = X::I, O = X::O, Size = Self>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) + Copy + Send;

    type Filter<X, H>: Xap<I = X::I, O = X::O, Size = Self::ThenBin>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send;

    type FilterMap<X, Q, H>: Xap<I = X::I, O = Q, Size = Self::ThenBin>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send;

    type FlatMap<X, V, H>: Xap<I = X::I, O = V::Item, Size = Many>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send;

    fn flat_map<X, V, H>(x: X, h: H) -> Self::FlatMap<X, V, H>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send;

    // transformations - helper

    type Mapped<X, M>: Xap<I = X::I, O = M::O, Size = Self>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;
}
