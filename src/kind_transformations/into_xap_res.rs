use crate::infallible::Xap;
use crate::infallible::fun::*;
use crate::infallible::sizes::{Bin, Many, One};
use crate::infallible::xap_variants::*;
use crate::result::XapRes;
use crate::result::size_pairs::IntoSizePair;

pub trait IntoXapRes<M, E>: Xap<O = Result<M, E>>
where
    Self::Size: IntoSizePair,
{
    fn into_xap_res(self) -> XapRes<M, E, Self, Id<M>, <Self::Size as IntoSizePair>::ThenOne>;
}

// bin_f

impl<M, E, X, G> IntoXapRes<M, E> for BinF<X, G>
where
    X: Xap<Size = Bin>,
    G: FilterMap<I = X::O, O = Result<M, E>>,
{
    // type XapRes = XapResBinOne<M, E, Self, Id<M>>;

    // fn into_xap_res(self) -> Self::XapRes {
    //     XapResBinOne::new(self, Id::new())
    // }

    fn into_xap_res(self) -> XapRes<M, E, Self, Id<M>, <Self::Size as IntoSizePair>::ThenOne> {
        XapRes::new(self, Id::new())
    }
}

// // bin_m

// impl<T, E, X, G> IntoXapRes for BinM<X, G>
// where
//     X: Xap<Size = Bin>,
//     G: Map<I = X::O, O = Result<T, E>>,
// {
//     type XapRes = XapResBinOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResBinOne::new(self, Id::new())
//     }
// }

// // bin_x

// impl<T, E, X, G> IntoXapRes for BinX<X, G>
// where
//     X: Xap<Size = Bin>,
//     G: FlatMap<I = X::O>,
//     G::O: IntoIterator<Item = Result<T, E>>,
// {
//     type XapRes = XapResManyOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResManyOne::new(self, Id::new())
//     }
// }

// // id

// impl<T, E> IntoXapRes for Id<Result<T, E>> {
//     type XapRes = XapResOneOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResOneOne::new(self, Id::new())
//     }
// }

// // many_f

// impl<T, E, X, G> IntoXapRes for ManyF<X, G>
// where
//     X: Xap<Size = Many>,
//     G: FilterMap<I = X::O, O = Result<T, E>>,
// {
//     type XapRes = XapResManyOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResManyOne::new(self, Id::new())
//     }
// }

// // many_m

// impl<T, E, X, G> IntoXapRes for ManyM<X, G>
// where
//     X: Xap<Size = Many>,
//     G: Map<I = X::O, O = Result<T, E>>,
// {
//     type XapRes = XapResManyOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResManyOne::new(self, Id::new())
//     }
// }

// // many_x

// impl<T, E, X, G> IntoXapRes for ManyX<X, G>
// where
//     X: Xap<Size = Many>,
//     G: FlatMap<I = X::O>,
//     G::O: IntoIterator<Item = Result<T, E>>,
// {
//     type XapRes = XapResManyOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResManyOne::new(self, Id::new())
//     }
// }

// // one_f

// impl<T, E, X, G> IntoXapRes for OneF<X, G>
// where
//     X: Xap<Size = One>,
//     G: FilterMap<I = X::O, O = Result<T, E>>,
// {
//     type XapRes = XapResBinOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResBinOne::new(self, Id::new())
//     }
// }

// // one_m

// impl<T, E, X, G> IntoXapRes for OneM<X, G>
// where
//     X: Xap<Size = One>,
//     G: Map<I = X::O, O = Result<T, E>>,
// {
//     type XapRes = XapResOneOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResOneOne::new(self, Id::new())
//     }
// }

// // one_x

// impl<T, E, X, G> IntoXapRes for OneX<X, G>
// where
//     X: Xap<Size = One>,
//     G: FlatMap<I = X::O>,
//     G::O: IntoIterator<Item = Result<T, E>>,
// {
//     type XapRes = XapResManyOne<T, E, Self, Id<T>>;

//     fn into_xap_res(self) -> Self::XapRes {
//         XapResManyOne::new(self, Id::new())
//     }
// }
