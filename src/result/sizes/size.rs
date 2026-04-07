use crate::infallible::fun::*;
use crate::infallible::sizes::Many;
use crate::infallible::sizes::Size;
use crate::result::XapRes;

pub trait SizeRes: Size {
    // transformations

    type ResMap<X, Q, H>: XapRes<I = X::I, O = Q, Size = Self>
    where
        X: XapRes<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    fn res_map<X, Q, H>(x: X, h: H) -> Self::ResMap<X, Q, H>
    where
        X: XapRes<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    type ResInspect<X, H>: XapRes<I = X::I, O = X::O, Size = Self>
    where
        X: XapRes<Size = Self>,
        H: Fn(&X::O) + Copy + Send;

    fn res_inspect<X, H>(x: X, h: H) -> Self::ResInspect<X, H>
    where
        X: XapRes<Size = Self>,
        H: Fn(&X::O) + Copy + Send;

    type ResFilter<X, H>: XapRes<I = X::I, O = X::O, Size = Self::ThenBin>
    where
        X: XapRes<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send;

    fn res_filter<X, H>(x: X, h: H) -> Self::ResFilter<X, H>
    where
        X: XapRes<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send;

    type ResFilterMap<X, Q, H>: XapRes<I = X::I, O = Q, Size = Self::ThenBin>
    where
        X: XapRes<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send;

    fn res_filter_map<X, Q, H>(x: X, h: H) -> Self::ResFilterMap<X, Q, H>
    where
        X: XapRes<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send;

    type ResFlatMap<X, V, H>: XapRes<I = X::I, O = V::Item, Size = Many>
    where
        X: XapRes<Size = Self>,
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send;

    fn res_flat_map<X, V, H>(x: X, h: H) -> Self::ResFlatMap<X, V, H>
    where
        X: XapRes<Size = Self>,
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send;

    // transformations - helper

    type ResMapped<X, M>: XapRes<I = X::I, O = M::O, Size = Self>
    where
        X: XapRes<Size = Self>,
        M: Map<I = X::O>;

    fn res_mapped<X, M>(x: X, m: M) -> Self::ResMapped<X, M>
    where
        X: XapRes<Size = Self>,
        M: Map<I = X::O>;
}
