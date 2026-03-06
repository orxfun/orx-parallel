use crate::xap::{
    stopper::{MightStop, StoppedBy},
    xap_trait::{IterOf, Xap},
};

pub struct MightStopMap<E, X: Xap<S = MightStop<E>>, O, F: Fn(X::O) -> O> {
    i: IterOf<X>,
    f: F,
}

impl<E, X: Xap<S = MightStop<E>>, O, F: Fn(X::O) -> O> MightStopMap<E, X, O, F> {
    pub fn new(i: IterOf<X>, f: F) -> Self {
        Self { i, f }
    }
}

impl<E, X: Xap<S = MightStop<E>>, O, F: Fn(X::O) -> O> Iterator for MightStopMap<E, X, O, F> {
    type Item = Result<O, StoppedBy<E>>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| x.map(|y| (self.f)(y)))
    }
}
