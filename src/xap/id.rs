use crate::xap::{joker::Joker, xap_trait::Xap};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;

pub struct Id<I>(PhantomData<I>);

impl<I> Id<I> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type Map<Q, G>
        = Joker<I, Q>
    where
        G: Fn(Self::O) -> Q;

    type Inspect<G>
        = Joker<I, I>
    where
        G: Fn(&Self::O);

    type Filter<G>
        = Joker<I, I>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>
        = Joker<I, Q>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>
        = Joker<I, V::Item>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;

    type Res = ();

    type ResOf<T> = T;

    #[inline(always)]
    fn push_to_vec_with_idx(
        &self,
        i: Self::I,
        idx: usize,
        vec: &mut Vec<(usize, Self::O)>,
    ) -> Self::Res {
        vec.push((idx, i));
    }

    #[inline(always)]
    fn push_to_pinned_vec<P: PinnedVec<Self::O>>(&self, i: Self::I, vec: &mut P) -> Self::Res {
        vec.push(i);
    }

    #[inline(always)]
    fn next_any(&self, i: Self::I) -> Self::ResOf<Option<Self::O>> {
        Some(i)
    }

    #[inline(always)]
    fn next(&self, i: Self::I) -> Self::ResOf<Option<Self::O>> {
        Some(i)
    }

    #[inline(always)]
    fn reduce<R>(&self, reduce: R, i: Self::I, acc: Option<Self::O>) -> Self::ResOf<Option<Self::O>>
    where
        R: Fn(Self::O, Self::O) -> Self::O,
    {
        Some(match acc {
            Some(acc) => reduce(acc, i),
            None => i,
        })
    }
}
