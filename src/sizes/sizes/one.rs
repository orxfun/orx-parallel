use crate::infallible::{Xap, fun::*, xap_variants::*};
use crate::infallible_use::{XapUse, fun::*, xap_variants::*};
use crate::sizes::{OneOne, Size, sizes::Bin};

#[derive(Clone, Copy, Default)]
pub struct One;

impl Size for One {
    type ThenBin = Bin;

    type IntoPair = OneOne;

    fn elem_len() -> Option<usize> {
        Some(1)
    }

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

    type Flatten<X>
        = OneX<X, FnFlatten<X::O>>
    where
        X: Xap<Size = Self>,
        X::O: IntoIterator;

    fn flatten<X>(x: X) -> Self::Flatten<X>
    where
        X: Xap<Size = Self>,
        X::O: IntoIterator,
    {
        OneX::new(x, FnFlatten::new())
    }

    type Mapped<X, M>
        = OneM<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>,
    {
        OneM::new(x, m)
    }

    // use transformations

    type UMap<X, Q, H>
        = UOneM<X, UFnMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send;

    fn u_map<X, Q, H>(x: X, h: H) -> Self::UMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Q + Copy + Send,
    {
        UOneM::new(x, UFnMap::new(h))
    }

    type UInspect<X, H>
        = UOneM<X, UFnIns<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send;

    fn u_inspect<X, H>(x: X, h: H) -> Self::UInspect<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) + Copy + Send,
    {
        UOneM::new(x, UFnIns::new(h))
    }

    type UFilter<X, H>
        = UOneF<X, UFnFil<X::U, X::O, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send;

    fn u_filter<X, H>(x: X, h: H) -> Self::UFilter<X, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, &X::O) -> bool + Copy + Send,
    {
        UOneF::new(x, UFnFil::new(h))
    }

    type UFilterMap<X, Q, H>
        = UOneF<X, UFnFilMap<X::U, X::O, Q, H>>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send;

    fn u_filter_map<X, Q, H>(x: X, h: H) -> Self::UFilterMap<X, Q, H>
    where
        X: XapUse<Size = Self>,
        H: Fn(&mut X::U, X::O) -> Option<Q> + Copy + Send,
    {
        UOneF::new(x, UFnFilMap::new(h))
    }

    type UFlatMap<X, V, H>
        = UOneX<X, UFnFlatMap<X::U, X::O, V, H>>
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
        UOneX::new(x, UFnFlatMap::new(h))
    }

    type UFlatten<X>
        = UOneX<X, UFnFlatten<X::U, X::O>>
    where
        X: XapUse<Size = Self>,
        X::O: IntoIterator;

    fn u_flatten<X>(x: X) -> Self::UFlatten<X>
    where
        X: XapUse<Size = Self>,
        X::O: IntoIterator,
    {
        UOneX::new(x, UFnFlatten::new())
    }

    type UMapped<X, M>
        = UOneM<X, M>
    where
        X: XapUse<Size = Self>,
        M: UMap<U = X::U, I = X::O>;

    fn u_mapped<X, M>(x: X, m: M) -> Self::UMapped<X, M>
    where
        X: XapUse<Size = Self>,
        M: UMap<U = X::U, I = X::O>,
    {
        UOneM::new(x, m)
    }
}
