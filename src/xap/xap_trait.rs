use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;

pub trait Xap {
    type I;

    type O;

    // transformations

    type Map<Q, G>: Xap<I = Self::I, O = Q>
    where
        G: Fn(Self::O) -> Q;

    type Inspect<G>: Xap<I = Self::I, O = Self::O>
    where
        G: Fn(&Self::O);

    type Filter<G>: Xap<I = Self::I, O = Self::O>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>: Xap<I = Self::I, O = Q>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>: Xap<I = Self::I, O = V::Item>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;

    // action results

    type Res;

    type ResOf<T>;

    // collect

    fn push_to_vec_with_idx(
        &self,
        i: Self::I,
        idx: usize,
        vec: &mut Vec<(usize, Self::O)>,
    ) -> Self::Res;

    fn push_to_pinned_vec<P: PinnedVec<Self::O>>(&self, i: Self::I, vector: &mut P) -> Self::Res;

    fn next_any(&self, i: Self::I) -> Self::ResOf<Self::O>;

    fn next(&self, i: Self::I) -> Self::ResOf<Self::O>;

    fn reduce<R>(
        &self,
        reduce: R,
        i: Self::I,
        acc: Option<Self::O>,
    ) -> Self::ResOf<Option<Self::O>>
    where
        R: Fn(Self::O, Self::O) -> Self::O;
}
