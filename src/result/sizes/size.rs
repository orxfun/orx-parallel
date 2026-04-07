use crate::infallible::FilMapOf;
use crate::infallible::FilOf;
use crate::infallible::FlatMapOf;
use crate::infallible::InsOf;
use crate::infallible::MapOf;
use crate::infallible::MappedOf;
use crate::infallible::fun::*;
use crate::infallible::sizes::Many;
use crate::infallible::sizes::Size;
use crate::result::XapRes;
use crate::result::xap_res::OutOf;

pub trait SizeRes: Size {
    // transformations

    type ResMap<X, Q, H>: XapRes<X1 = X::X1, X2 = MapOf<X::X2, Q, H>, Size = Self>
    where
        X: XapRes<Size = Self>,
        H: Fn(OutOf<X>) -> Q + Copy + Send;

    fn res_map<X, Q, H>(x: X, h: H) -> Self::ResMap<X, Q, H>
    where
        X: XapRes<Size = Self>,
        H: Fn(OutOf<X>) -> Q + Copy + Send;

    type ResInspect<X, H>: XapRes<X1 = X::X1, X2 = InsOf<X::X2, H>, Size = Self>
    where
        X: XapRes<Size = Self>,
        H: Fn(&OutOf<X>) + Copy + Send;

    fn res_inspect<X, H>(x: X, h: H) -> Self::ResInspect<X, H>
    where
        X: XapRes<Size = Self>,
        H: Fn(&OutOf<X>) + Copy + Send;

    type ResFilter<X, H>: XapRes<X1 = X::X1, X2 = FilOf<X::X2, H>, Size = Self::ThenBin>
    where
        X: XapRes<Size = Self>,
        H: Fn(&OutOf<X>) -> bool + Copy + Send;

    fn res_filter<X, H>(x: X, h: H) -> Self::ResFilter<X, H>
    where
        X: XapRes<Size = Self>,
        H: Fn(&OutOf<X>) -> bool + Copy + Send;

    type ResFilterMap<X, Q, H>: XapRes<X1 = X::X1, X2 = FilMapOf<X::X2, Q, H>, Size = Self::ThenBin>
    where
        X: XapRes<Size = Self>,
        H: Fn(OutOf<X>) -> Option<Q> + Copy + Send;

    fn res_filter_map<X, Q, H>(x: X, h: H) -> Self::ResFilterMap<X, Q, H>
    where
        X: XapRes<Size = Self>,
        H: Fn(OutOf<X>) -> Option<Q> + Copy + Send;

    type ResFlatMap<X, V, H>: XapRes<X1 = X::X1, X2 = FlatMapOf<X::X2, V, H>, Size = Many>
    where
        X: XapRes<Size = Self>,
        V: IntoIterator,
        H: Fn(OutOf<X>) -> V + Copy + Send;

    fn res_flat_map<X, V, H>(x: X, h: H) -> Self::ResFlatMap<X, V, H>
    where
        X: XapRes<Size = Self>,
        V: IntoIterator,
        H: Fn(OutOf<X>) -> V + Copy + Send;

    // transformations - helper

    type ResMapped<X, M>: XapRes<X1 = X::X1, X2 = MappedOf<X::X2, M>, Size = Self>
    where
        X: XapRes<Size = Self>,
        M: Map<I = OutOf<X>>;

    fn res_mapped<X, M>(x: X, m: M) -> Self::ResMapped<X, M>
    where
        X: XapRes<Size = Self>,
        M: Map<I = OutOf<X>>;
}
