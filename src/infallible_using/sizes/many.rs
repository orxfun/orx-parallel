use crate::infallible_using::Xap;
use crate::infallible_using::fun::*;
use crate::infallible_using::sizes::SizeInf;
use crate::infallible_using::xap_variants::*;
use crate::sizes::Many;

impl SizeInf for Many {
    // transformations

    type Map<X, Q, H>
        = ManyM<X, FnMap<X::U, X::O, Q, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send,
    {
        ManyM::new(x, FnMap::new(h))
    }

    type Inspect<X, H>
        = ManyM<X, FnIns<X::U, X::O, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send,
    {
        ManyM::new(x, FnIns::new(h))
    }

    type Filter<X, H>
        = ManyF<X, FnFil<X::U, X::O, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send,
    {
        ManyF::new(x, FnFil::new(h))
    }

    type FilterMap<X, Q, H>
        = ManyF<X, FnFilMap<X::U, X::O, Q, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send,
    {
        ManyF::new(x, FnFilMap::new(h))
    }

    type FlatMap<X, V, H>
        = ManyX<X, FnFlatMap<X::U, X::O, V, H>>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send;

    fn flat_map<X, V, H>(x: X, h: H) -> Self::FlatMap<X, V, H>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(&mut X::U, X::O) -> V + Copy + Send,
    {
        ManyX::new(x, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<X, M>
        = ManyM<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<U = X::U, I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<U = X::U, I = X::O>,
    {
        ManyM::new(x, m)
    }
}
