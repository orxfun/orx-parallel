use crate::infallible_use::XapUse;
use crate::infallible_use::fun::*;
use crate::infallible_use::sizes::SizeInfUse;
use crate::infallible_use::xap_variants::*;
use crate::sizes::Bin;

impl SizeInfUse for Bin {
    // transformations

    type UMap<X, Q, H>
        = BinM<X, FnMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn u_map<X, Q, H>(x: X, h: H) -> Self::UMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send,
    {
        BinM::new(x, FnMap::new(h))
    }

    type UInspect<X, H>
        = BinM<X, FnIns<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn u_inspect<X, H>(x: X, h: H) -> Self::UInspect<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send,
    {
        BinM::new(x, FnIns::new(h))
    }

    type UFilter<X, H>
        = BinF<X, FnFil<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn u_filter<X, H>(x: X, h: H) -> Self::UFilter<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send,
    {
        BinF::new(x, FnFil::new(h))
    }

    type UFilterMap<X, Q, H>
        = BinF<X, FnFilMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn u_filter_map<X, Q, H>(x: X, h: H) -> Self::UFilterMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send,
    {
        BinF::new(x, FnFilMap::new(h))
    }

    type UFlatMap<X, V, H>
        = BinX<X, FnFlatMap<X::U, X::O, V, H>>
    where
        X: XapUse<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    fn u_flat_map<X, V, H>(x: X, h: H) -> Self::UFlatMap<X, V, H>
    where
        X: XapUse<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send,
    {
        BinX::new(x, FnFlatMap::new(h))
    }

    // transformations - helper

    type UMapped<X, M>
        = BinM<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>;

    fn u_mapped<X, M>(x: X, m: M) -> Self::UMapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>,
    {
        BinM::new(x, m)
    }
}
