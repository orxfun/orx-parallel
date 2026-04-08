use crate::infallible_using::Xap;
use crate::infallible_using::fun::*;
use crate::sizes::Many;
use crate::sizes::Size;

pub trait SizeInf: Size {
    // transformations

    // type Map<U, X, Q, H>: Xap<I = X::I, O = Q, Size = Self>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     H: Fn(&mut U, X::O) -> Q + Copy + Send;

    // fn map<U, X, Q, H>(x: X, h: H) -> Self::Map<U, X, Q, H>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     H: Fn(&mut U, X::O) -> Q + Copy + Send;

    // type Inspect<U, X, H>: Xap<I = X::I, O = X::O, Size = Self>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     H: Fn(&mut U, &X::O) + Copy + Send;

    // fn inspect<U, X, H>(x: X, h: H) -> Self::Inspect<U, X, H>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     H: Fn(&mut U, &X::O) + Copy + Send;

    // type Filter<U, X, H>: Xap<I = X::I, O = X::O, Size = Self::ThenBin>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     H: Fn(&mut U, &X::O) -> bool + Copy + Send;

    // fn filter<U, X, H>(x: X, h: H) -> Self::Filter<U, X, H>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     H: Fn(&mut U, &X::O) -> bool + Copy + Send;

    // type FilterMap<U, X, Q, H>: Xap<I = X::I, O = Q, Size = Self::ThenBin>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     H: Fn(&mut U, X::O) -> Option<Q> + Copy + Send;

    // fn filter_map<U, X, Q, H>(x: X, h: H) -> Self::FilterMap<U, X, Q, H>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     H: Fn(&mut U, X::O) -> Option<Q> + Copy + Send;

    // type FlatMap<U, X, V, H>: Xap<I = X::I, O = V::Item, Size = Many>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     V: IntoIterator,
    //     H: Fn(&mut U, X::O) -> V + Copy + Send;

    // fn flat_map<U, X, V, H>(x: X, h: H) -> Self::FlatMap<U, X, V, H>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     V: IntoIterator,
    //     H: Fn(&mut U, X::O) -> V + Copy + Send;

    // // transformations - helper

    // type Mapped<U, X, M>: Xap<I = X::I, O = M::O, Size = Self>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     M: Map<U = U, I = X::O>;

    // fn mapped<U, X, M>(x: X, m: M) -> Self::Mapped<U, X, M>
    // where
    //     X: Xap<U = U, Size = Self>,
    //     M: Map<U = U, I = X::O>;
}
