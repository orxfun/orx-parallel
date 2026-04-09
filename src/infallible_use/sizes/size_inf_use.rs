use crate::infallible_use::XapUse;
use crate::infallible_use::fun::*;
use crate::sizes::Many;
use crate::sizes::Size;

pub trait SizeInfUse: Size {
    // transformations

    type Map<X, Q, H>: XapUse<U = X::U, I = X::I, O = Q, Size = Self>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    type Inspect<X, H>: XapUse<U = X::U, I = X::I, O = X::O, Size = Self>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    type Filter<X, H>: XapUse<U = X::U, I = X::I, O = X::O, Size = Self::ThenBin>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    type FilterMap<X, Q, H>: XapUse<U = X::U, I = X::I, O = Q, Size = Self::ThenBin>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    type FlatMap<X, V, H>: XapUse<U = X::U, I = X::I, O = V::Item, Size = Many>
    where
        X: XapUse<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    fn flat_map<X, V, H>(x: X, h: H) -> Self::FlatMap<X, V, H>
    where
        X: XapUse<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    // transformations - helper

    type Mapped<X, M>: XapUse<U = X::U, I = X::I, O = M::O, Size = Self>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>;
}
