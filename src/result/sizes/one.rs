use crate::infallible::MapOf;
use crate::infallible::sizes::One;
use crate::result::xap_res_variants::*;
use crate::result::{XapRes, sizes::size::SizeRes};

// impl SizeRes for One {
//     type ResMap<X, Q, H>
//         = XapResOneOne<M, E, X1, MapOf<X2, Q, H>>
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
// }
