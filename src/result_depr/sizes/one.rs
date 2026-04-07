use crate::infallible::MapOf;
use crate::infallible::sizes::One;
use crate::result::xap_res::OutOf;
use crate::result::xap_res_variants::*;
use crate::result::{XapRes, sizes::size::SizeRes};

impl SizeRes for One {
    type ResMap<X, Q, H>
        = XapResOneOne<X::M, X::E, X::X1, MapOf<X::X2, Q, H>>
    where
        X: XapRes<Size = Self>,
        H: Fn(OutOf<X>) -> Q + Copy + Send;

    // fn res_map<X, Q, H>(x: X, h: H) -> Self::ResMap<X, Q, H>
    // where
    //     X: XapRes<Size = Self>,
    //     H: Fn(OutOf<X>) -> Q + Copy + Send,
    // {
    //     XapResOneOne::new(x.x1, x.x2.map(h))
    // }

    // type ResInspect<X, H>
    // where
    //     X: XapRes<Size = Self>,
    //     H: Fn(&OutOf<X>) + Copy + Send;

    // fn res_inspect<X, H>(x: X, h: H) -> Self::ResInspect<X, H>
    // where
    //     X: XapRes<Size = Self>,
    //     H: Fn(&OutOf<X>) + Copy + Send,
    // {
    //     todo!()
    // }

    // type ResFilter<X, H>
    // where
    //     X: XapRes<Size = Self>,
    //     H: Fn(&OutOf<X>) -> bool + Copy + Send;

    // fn res_filter<X, H>(x: X, h: H) -> Self::ResFilter<X, H>
    // where
    //     X: XapRes<Size = Self>,
    //     H: Fn(&OutOf<X>) -> bool + Copy + Send,
    // {
    //     todo!()
    // }

    // type ResFilterMap<X, Q, H>
    // where
    //     X: XapRes<Size = Self>,
    //     H: Fn(OutOf<X>) -> Option<Q> + Copy + Send;

    // fn res_filter_map<X, Q, H>(x: X, h: H) -> Self::ResFilterMap<X, Q, H>
    // where
    //     X: XapRes<Size = Self>,
    //     H: Fn(OutOf<X>) -> Option<Q> + Copy + Send,
    // {
    //     todo!()
    // }

    // type ResFlatMap<X, V, H>
    // where
    //     X: XapRes<Size = Self>,
    //     V: IntoIterator,
    //     H: Fn(OutOf<X>) -> V + Copy + Send;

    // fn res_flat_map<X, V, H>(x: X, h: H) -> Self::ResFlatMap<X, V, H>
    // where
    //     X: XapRes<Size = Self>,
    //     V: IntoIterator,
    //     H: Fn(OutOf<X>) -> V + Copy + Send,
    // {
    //     todo!()
    // }

    // type ResMapped<X, M>
    // where
    //     X: XapRes<Size = Self>,
    //     M: crate::infallible::fun::Map<I = OutOf<X>>;

    // fn res_mapped<X, M>(x: X, m: M) -> Self::ResMapped<X, M>
    // where
    //     X: XapRes<Size = Self>,
    //     M: crate::infallible::fun::Map<I = OutOf<X>>,
    // {
    //     todo!()
    // }
}
