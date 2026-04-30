use crate::infallible::{Xap, fun::*, xap_variants::*};
use crate::infallible_use::{XapUse, fun::*, xap_variants::*};
use crate::sizes::{ManyOne, Size};

#[derive(Clone, Copy, Default)]
pub struct Many;

impl Size for Many {
    type ThenBin = Many;

    type IntoPair = ManyOne;

    fn elem_len() -> Option<usize> {
        None
    }

    // transformations

    type Map<X, Q, H>
        = ManyM<X, FnMap<X::O, Q, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send,
    {
        ManyM::new(x, FnMap::new(h))
    }

    type Inspect<X, H>
        = ManyM<X, FnIns<X::O, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) + Copy + Send,
    {
        ManyM::new(x, FnIns::new(h))
    }

    type Filter<X, H>
        = ManyF<X, FnFil<X::O, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send,
    {
        ManyF::new(x, FnFil::new(h))
    }

    type FilterMap<X, Q, H>
        = ManyF<X, FnFilMap<X::O, Q, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send,
    {
        ManyF::new(x, FnFilMap::new(h))
    }

    type FlatMap<X, V, H>
        = ManyX<X, FnFlatMap<X::O, V, H>>
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
        ManyX::new(x, FnFlatMap::new(h))
    }

    type Flatten<X>
        = ManyX<X, FnFlatten<X::O>>
    where
        X: Xap<Size = Self>,
        X::O: IntoIterator;

    fn flatten<X>(x: X) -> Self::Flatten<X>
    where
        X: Xap<Size = Self>,
        X::O: IntoIterator,
    {
        ManyX::new(x, FnFlatten::new())
    }

    type Mapped<X, M>
        = ManyM<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>,
    {
        ManyM::new(x, m)
    }

    // use transformations

    type UMap<X, Q, H>
        = UManyM<X, UFnMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn u_map<X, Q, H>(x: X, h: H) -> Self::UMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send,
    {
        UManyM::new(x, UFnMap::new(h))
    }

    type UInspect<X, H>
        = UManyM<X, UFnIns<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn u_inspect<X, H>(x: X, h: H) -> Self::UInspect<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send,
    {
        UManyM::new(x, UFnIns::new(h))
    }

    type UFilter<X, H>
        = UManyF<X, UFnFil<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn u_filter<X, H>(x: X, h: H) -> Self::UFilter<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send,
    {
        UManyF::new(x, UFnFil::new(h))
    }

    type UFilterMap<X, Q, H>
        = UManyF<X, UFnFilMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn u_filter_map<X, Q, H>(x: X, h: H) -> Self::UFilterMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send,
    {
        UManyF::new(x, UFnFilMap::new(h))
    }

    type UFlatMap<X, V, H>
        = UManyX<X, UFnFlatMap<X::U, X::O, V, H>>
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
        UManyX::new(x, UFnFlatMap::new(h))
    }

    type UFlatten<X>
        = UManyX<X, UFnFlatten<X::U, X::O>>
    where
        X: XapUse<Size = Self>,
        X::O: IntoIterator;

    fn u_flatten<X>(x: X) -> Self::UFlatten<X>
    where
        X: XapUse<Size = Self>,
        X::O: IntoIterator,
    {
        UManyX::new(x, UFnFlatten::new())
    }

    type UMapped<X, M>
        = UManyM<X, M>
    where
        X: XapUse<Size = Self>,
        M: UMap<U = X::U, I = X::O>;

    fn u_mapped<X, M>(x: X, m: M) -> Self::UMapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: UMap<U = X::U, I = X::O>,
    {
        UManyM::new(x, m)
    }
}
