use crate::infallible_use::XapUse;
use crate::infallible_use::fun::*;
use crate::sizes::Many;
use crate::sizes::Size;

pub trait SizeInfUse: Size {
    // transformations

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

    // transformations - helper

    type UMapped<X, M>: XapUse<U = X::U, I = X::I, O = M::O, Size = Self>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>;

    fn u_mapped<X, M>(x: X, m: M) -> Self::UMapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>;
}
