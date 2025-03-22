use super::values::Values;
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct Mfm<I, T, Vt, O, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values<Item = T>,
    O: Send + Sync,
    Vo: Values<Item = O>,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
{
    params: Params,
    iter: I,
    map1: M1,
    filter: F,
    map2: M2,
    phantom: PhantomData<(T, O, Vo)>,
}

impl<I, T, Vt, O, Vo, M1, F, M2> Mfm<I, T, Vt, O, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values<Item = T>,
    O: Send + Sync,
    Vo: Values<Item = O>,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
{
    pub fn new(params: Params, iter: I, map1: M1, filter: F, map2: M2) -> Self {
        Self {
            params,
            iter,
            map1,
            filter,
            map2,
            phantom: PhantomData,
        }
    }

    pub fn destruct(self) -> (Params, I, M1, F, M2) {
        (self.params, self.iter, self.map1, self.filter, self.map2)
    }

    pub fn params(&self) -> &Params {
        &self.params
    }
}
