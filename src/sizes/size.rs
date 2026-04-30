use crate::infallible::Xap;
use crate::infallible::fun::Map;
use crate::infallible_use::XapUse;
use crate::infallible_use::fun::UMap;
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

    type Flatten<X>: Xap<I = X::I, O = <X::O as IntoIterator>::Item, Size = Many>
    where
        X: Xap<Size = Self>,
        X::O: IntoIterator;

    fn flatten<X>(x: X) -> Self::Flatten<X>
    where
        X: Xap<Size = Self>,
        X::O: IntoIterator;

    type Mapped<X, M>: Xap<I = X::I, O = M::O, Size = Self>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;

    // use transformations

    type UMap<X, Q, H>: XapUse<U = X::U, I = X::I, O = Q, Size = Self>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn u_map<X, Q, H>(x: X, h: H) -> Self::UMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    type UInspect<X, H>: XapUse<U = X::U, I = X::I, O = X::O, Size = Self>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn u_inspect<X, H>(x: X, h: H) -> Self::UInspect<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    type UFilter<X, H>: XapUse<U = X::U, I = X::I, O = X::O, Size = Self::ThenBin>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn u_filter<X, H>(x: X, h: H) -> Self::UFilter<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    type UFilterMap<X, Q, H>: XapUse<U = X::U, I = X::I, O = Q, Size = Self::ThenBin>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn u_filter_map<X, Q, H>(x: X, h: H) -> Self::UFilterMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    type UFlatMap<X, V, H>: XapUse<U = X::U, I = X::I, O = V::Item, Size = Many>
    where
        X: XapUse<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    fn u_flat_map<X, V, H>(x: X, h: H) -> Self::UFlatMap<X, V, H>
    where
        X: XapUse<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    type UFlatten<X>: XapUse<U = X::U, I = X::I, O = <X::O as IntoIterator>::Item, Size = Many>
    where
        X: XapUse<Size = Self>,
        X::O: IntoIterator;

    fn u_flatten<X>(x: X) -> Self::UFlatten<X>
    where
        X: XapUse<Size = Self>,
        X::O: IntoIterator;

    type UMapped<X, M>: XapUse<U = X::U, I = X::I, O = M::O, Size = Self>
    where
        X: XapUse<Size = Self>,
        M: UMap<U = X::U, I = X::O>;

    fn u_mapped<X, M>(x: X, m: M) -> Self::UMapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: UMap<U = X::U, I = X::O>;
}
