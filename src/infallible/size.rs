use crate::infallible::Xap;
use crate::infallible::fun::*;
use crate::infallible::xap_variants::*;

pub trait Size {
    type ThenBin: Size;

    // transformations

    type Map<X, Q, H>: Xap<I = X::I, O = Q, Size = Self>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    type Inspect<X, H>: Xap<I = X::I, O = X::O, Size = Self>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) + Copy + Send;

    fn inspect<X, H>(x: X, h: H) -> Self::Inspect<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) + Copy + Send;

    type Filter<X, H>: Xap<I = X::I, O = X::O, Size = Self::ThenBin>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send;

    fn filter<X, H>(x: X, h: H) -> Self::Filter<X, H>
    where
        X: Xap<Size = Self>,
        H: Fn(&X::O) -> bool + Copy + Send;

    type FilterMap<X, Q, H>: Xap<I = X::I, O = Q, Size = Self::ThenBin>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send;

    fn filter_map<X, Q, H>(x: X, h: H) -> Self::FilterMap<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Option<Q> + Copy + Send;

    type FlatMap<X, V, H>: Xap<I = X::I, O = V::Item, Size = Many>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send;

    fn flat_map<X, V, H>(x: X, h: H) -> Self::FlatMap<X, V, H>
    where
        X: Xap<Size = Self>,
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send;

    // transformations - helper

    type Mapped<X, M>: Xap<I = X::I, O = M::O, Size = Self>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>;
}

// ONE

pub struct One;

impl Size for One {
    type ThenBin = Bin;

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

    fn mapped<X, M>(x: X, m: M) -> Self::Mapped<X, M>
    where
        X: Xap<Size = Self>,
        M: Map<I = X::O>,
    {
        OneM::new(x, m)
    }
}

// BIN

pub struct Bin;

impl Size for Bin {
    type ThenBin = Bin;

    // transformations

    type Map<X, Q, H>
        = BinM<X, FnMap<X::O, Q, H>>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send;

    fn map<X, Q, H>(x: X, h: H) -> Self::Map<X, Q, H>
    where
        X: Xap<Size = Self>,
        H: Fn(X::O) -> Q + Copy + Send,
    {
        BinM::new(x, FnMap::new(h))
    }
}

// MANY

pub struct Many;

impl Size for Many {
    type ThenBin = Many;

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
}
