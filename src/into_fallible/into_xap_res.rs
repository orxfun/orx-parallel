use crate::infallible::xap_variants::*;
use crate::result::xap_res::XapRes;
use crate::result::xap_res_variants::*;

pub trait IntoXapRes {
    type XapRes: XapRes;

    fn into_xap_res(self) -> Self::XapRes;
}

// bin_f

// pub struct BinF<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> {
//     x: X,
//     g: G,
// }

// id

impl<T, E> IntoXapRes for Id<Result<T, E>> {
    type XapRes = XapResOneOne<T, E, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapRes {
        XapResOneOne::new(self, Id::new())
    }
}
