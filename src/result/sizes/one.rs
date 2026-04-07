use crate::infallible::sizes::One;
use crate::result::{XapRes, sizes::size::SizeRes};

// impl SizeRes for One {
//     type ResMap<X, Q, H>
//     where
//         X: XapRes<Size = Self>,
//         H: Fn(X::O) -> Q + Copy + Send;

//     fn res_map<X, Q, H>(x: X, h: H) -> Self::ResMap<X, Q, H>
//     where
//         X: XapRes<Size = Self>,
//         H: Fn(X::O) -> Q + Copy + Send,
//     {
//         todo!()
//     }

//     type ResInspect<X, H>
//     where
//         X: XapRes<Size = Self>,
//         H: Fn(&X::O) + Copy + Send;

//     fn res_inspect<X, H>(x: X, h: H) -> Self::ResInspect<X, H>
//     where
//         X: XapRes<Size = Self>,
//         H: Fn(&X::O) + Copy + Send,
//     {
//         todo!()
//     }

//     type ResFilter<X, H>
//     where
//         X: XapRes<Size = Self>,
//         H: Fn(&X::O) -> bool + Copy + Send;

//     fn res_filter<X, H>(x: X, h: H) -> Self::ResFilter<X, H>
//     where
//         X: XapRes<Size = Self>,
//         H: Fn(&X::O) -> bool + Copy + Send,
//     {
//         todo!()
//     }

//     type ResFilterMap<X, Q, H>
//     where
//         X: XapRes<Size = Self>,
//         H: Fn(X::O) -> Option<Q> + Copy + Send;

//     fn res_filter_map<X, Q, H>(x: X, h: H) -> Self::ResFilterMap<X, Q, H>
//     where
//         X: XapRes<Size = Self>,
//         H: Fn(X::O) -> Option<Q> + Copy + Send,
//     {
//         todo!()
//     }

//     type ResFlatMap<X, V, H>
//     where
//         X: XapRes<Size = Self>,
//         V: IntoIterator,
//         H: Fn(X::O) -> V + Copy + Send;

//     fn res_flat_map<X, V, H>(x: X, h: H) -> Self::ResFlatMap<X, V, H>
//     where
//         X: XapRes<Size = Self>,
//         V: IntoIterator,
//         H: Fn(X::O) -> V + Copy + Send,
//     {
//         todo!()
//     }

//     type ResMapped<X, M>
//     where
//         X: XapRes<Size = Self>,
//         M: crate::infallible::fun::Map<I = X::O>;

//     fn res_mapped<X, M>(x: X, m: M) -> Self::ResMapped<X, M>
//     where
//         X: XapRes<Size = Self>,
//         M: crate::infallible::fun::Map<I = X::O>,
//     {
//         todo!()
//     }
// }
