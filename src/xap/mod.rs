use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;

pub trait Xap {
    type I;

    type O;

    // transformations

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
