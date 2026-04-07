use crate::infallible::Xap;
use crate::infallible::fun::*;
use crate::infallible::sizes::{Bin, Many, One};
use crate::infallible::xap_variants::*;
use crate::option::XapOpt;
use crate::option::xap_opt_variants::*;

pub trait IntoXapOpt: Xap {
    type XapOpt: XapOpt<I = Self::I>;

    fn into_xap_res(self) -> Self::XapOpt;
}

// bin_f

impl<T, X, G> IntoXapOpt for BinF<X, G>
where
    X: Xap<Size = Bin>,
    G: FilterMap<I = X::O, O = Option<T>>,
{
    type XapOpt = XapOptBinOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptBinOne::new(self, Id::new())
    }
}

// bin_m

impl<T, X, G> IntoXapOpt for BinM<X, G>
where
    X: Xap<Size = Bin>,
    G: Map<I = X::O, O = Option<T>>,
{
    type XapOpt = XapOptBinOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptBinOne::new(self, Id::new())
    }
}

// bin_x

impl<T, X, G> IntoXapOpt for BinX<X, G>
where
    X: Xap<Size = Bin>,
    G: FlatMap<I = X::O>,
    G::O: IntoIterator<Item = Option<T>>,
{
    type XapOpt = XapOptManyOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptManyOne::new(self, Id::new())
    }
}

// id

impl<T> IntoXapOpt for Id<Option<T>> {
    type XapOpt = XapOptOneOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptOneOne::new(self, Id::new())
    }
}

// many_f

impl<T, X, G> IntoXapOpt for ManyF<X, G>
where
    X: Xap<Size = Many>,
    G: FilterMap<I = X::O, O = Option<T>>,
{
    type XapOpt = XapOptManyOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptManyOne::new(self, Id::new())
    }
}

// many_m

impl<T, X, G> IntoXapOpt for ManyM<X, G>
where
    X: Xap<Size = Many>,
    G: Map<I = X::O, O = Option<T>>,
{
    type XapOpt = XapOptManyOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptManyOne::new(self, Id::new())
    }
}

// many_x

impl<T, X, G> IntoXapOpt for ManyX<X, G>
where
    X: Xap<Size = Many>,
    G: FlatMap<I = X::O>,
    G::O: IntoIterator<Item = Option<T>>,
{
    type XapOpt = XapOptManyOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptManyOne::new(self, Id::new())
    }
}

// one_f

impl<T, X, G> IntoXapOpt for OneF<X, G>
where
    X: Xap<Size = One>,
    G: FilterMap<I = X::O, O = Option<T>>,
{
    type XapOpt = XapOptBinOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptBinOne::new(self, Id::new())
    }
}

// one_m

impl<T, X, G> IntoXapOpt for OneM<X, G>
where
    X: Xap<Size = One>,
    G: Map<I = X::O, O = Option<T>>,
{
    type XapOpt = XapOptOneOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptOneOne::new(self, Id::new())
    }
}

// one_x

impl<T, X, G> IntoXapOpt for OneX<X, G>
where
    X: Xap<Size = One>,
    G: FlatMap<I = X::O>,
    G::O: IntoIterator<Item = Option<T>>,
{
    type XapOpt = XapOptManyOne<T, Self, Id<T>>;

    fn into_xap_res(self) -> Self::XapOpt {
        XapOptManyOne::new(self, Id::new())
    }
}
