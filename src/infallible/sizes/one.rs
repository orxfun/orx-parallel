use crate::infallible::Xap;
use crate::infallible::fun::*;
use crate::infallible::sizes::size_inf::SizeInf;
use crate::infallible::xap_variants::*;
use crate::sizes::One;

impl SizeInf for One {
    // transformations

    type Map<X, Q, H>
        = OneM<X, FnMap<X::O, Q, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send,
    {
        OneM::new(x, FnMap::new(h))
    }

    type Inspect<X, H>
        = OneM<X, FnIns<X::O, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) + Copy + Send,
    {
        OneM::new(x, FnIns::new(h))
    }

    type Filter<X, H>
        = OneF<X, FnFil<X::O, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send,
    {
        OneF::new(x, FnFil::new(h))
    }

    type FilterMap<X, Q, H>
        = OneF<X, FnFilMap<X::O, Q, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(x, FnFilMap::new(h))
    }

    type FlatMap<X, V, H>
        = OneX<X, FnFlatMap<X::O, V, H>>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send;

    fn flat_map<X, V, H>(x: X, h: H) -> Self::FlatMap<X, V, H>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send,
    {
        OneX::new(x, FnFlatMap::new(h))
    }

    type Mapped<X, M>
        = OneM<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;

    // transformations - helper

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>,
    {
        OneM::new(x, m)
    }
}
