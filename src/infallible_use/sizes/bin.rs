use crate::infallible_use::XapUse;
use crate::infallible_use::fun::*;
use crate::infallible_use::sizes::SizeInfUse;
use crate::infallible_use::xap_variants::*;
use crate::sizes::Bin;

impl SizeInfUse for Bin {
    // transformations

    type Map<X, Q, H>
        = BinM<X, FnMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send,
    {
        BinM::new(x, FnMap::new(h))
    }

    type Inspect<X, H>
        = BinM<X, FnIns<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send,
    {
        BinM::new(x, FnIns::new(h))
    }

    type Filter<X, H>
        = BinF<X, FnFil<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send,
    {
        BinF::new(x, FnFil::new(h))
    }

    type FilterMap<X, Q, H>
        = BinF<X, FnFilMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send,
    {
        BinF::new(x, FnFilMap::new(h))
    }

    type FlatMap<X, V, H>
        = BinX<X, FnFlatMap<X::U, X::O, V, H>>
    where
        X: XapUse<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    fn flat_map<X, V, H>(x: X, h: H) -> Self::FlatMap<X, V, H>
    where
        X: XapUse<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send,
    {
        BinX::new(x, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<X, M>
        = BinM<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>,
    {
        BinM::new(x, m)
    }
}
