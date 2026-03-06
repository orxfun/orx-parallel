use crate::xap::stopper::{MightStopItem, StoppedBy};

pub struct MightStopMap<T, E, I: Iterator<Item = MightStopItem<T, E>>, O, F: Fn(T) -> O> {
    i: I,
    f: F,
}

impl<T, E, I: Iterator<Item = MightStopItem<T, E>>, O, F: Fn(T) -> O> MightStopMap<T, E, I, O, F> {
    pub fn new(i: I, f: F) -> Self {
        Self { i, f }
    }
}

impl<T, E, I: Iterator<Item = MightStopItem<T, E>>, O, F: Fn(T) -> O> Iterator
    for MightStopMap<T, E, I, O, F>
{
    type Item = Result<O, StoppedBy<E>>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| x.map(|y| (self.f)(y)))
    }
}
