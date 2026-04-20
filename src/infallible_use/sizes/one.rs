use crate::infallible_use::XapUse;
use crate::infallible_use::fun::*;
use crate::infallible_use::sizes::SizeInfUse;
use crate::infallible_use::xap_variants::*;
use crate::sizes::One;

impl SizeInfUse for One {
    // transformations

    type UMap<X, Q, H>
        = OneM<X, FnMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn u_map<X, Q, H>(x: X, h: H) -> Self::UMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send,
    {
        OneM::new(x, FnMap::new(h))
    }

    type UInspect<X, H>
        = OneM<X, FnIns<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn u_inspect<X, H>(x: X, h: H) -> Self::UInspect<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send,
    {
        OneM::new(x, FnIns::new(h))
    }

    type UFilter<X, H>
        = OneF<X, FnFil<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn u_filter<X, H>(x: X, h: H) -> Self::UFilter<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send,
    {
        OneF::new(x, FnFil::new(h))
    }

    type UFilterMap<X, Q, H>
        = OneF<X, FnFilMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn u_filter_map<X, Q, H>(x: X, h: H) -> Self::UFilterMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(x, FnFilMap::new(h))
    }

    type UFlatMap<X, V, H>
        = OneX<X, FnFlatMap<X::U, X::O, V, H>>
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
        OneX::new(x, FnFlatMap::new(h))
    }

    type UMapped<X, M>
        = OneM<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>;

    // transformations - helper

    fn u_mapped<X, M>(x: X, m: M) -> Self::UMapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>,
    {
        OneM::new(x, m)
    }
}
