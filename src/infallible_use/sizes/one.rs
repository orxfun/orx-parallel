use crate::infallible_use::XapUse;
use crate::infallible_use::fun::*;
use crate::infallible_use::sizes::SizeInf;
use crate::infallible_use::xap_variants::*;
use crate::sizes::One;

impl SizeInf for One {
    // transformations

    type Map<X, Q, H>
        = OneM<X, FnMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send,
    {
        OneM::new(x, FnMap::new(h))
    }

    type Inspect<X, H>
        = OneM<X, FnIns<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send,
    {
        OneM::new(x, FnIns::new(h))
    }

    type Filter<X, H>
        = OneF<X, FnFil<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send,
    {
        OneF::new(x, FnFil::new(h))
    }

    type FilterMap<X, Q, H>
        = OneF<X, FnFilMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(x, FnFilMap::new(h))
    }

    type FlatMap<X, V, H>
        = OneX<X, FnFlatMap<X::U, X::O, V, H>>
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
        OneX::new(x, FnFlatMap::new(h))
    }

    type Mapped<X, M>
        = OneM<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>;

    // transformations - helper

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: Map<U = X::U, I = X::O>,
    {
        OneM::new(x, m)
    }
}
