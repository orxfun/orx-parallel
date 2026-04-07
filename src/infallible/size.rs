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
