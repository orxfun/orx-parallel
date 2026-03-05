use crate::xap::xap_trait::Xap;
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;

pub struct Id<I>(PhantomData<I>);

impl<I> Id<I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

// impl<I> Xap for Id<I> {
//     type I = I;

//     type O = I;

//     type Map<Q, G>
//     where
//         G: Fn(Self::O) -> Q;

//     type Inspect<G>
//     where
//         G: Fn(&Self::O);

//     type Filter<G>
//     where
//         G: Fn(&Self::O) -> bool;

//     type FilterMap<Q, G>
//     where
//         G: Fn(Self::O) -> Option<Q>;

//     type FlatMap<V, G>
//     where
//         V: IntoIterator,
//         G: Fn(Self::O) -> V;

//     type Res;

//     type ResOf<T>;

//     fn push_to_vec_with_idx(
//         &self,
//         i: Self::I,
//         idx: usize,
//         vec: &mut Vec<(usize, Self::O)>,
//     ) -> Self::Res {
//         todo!()
//     }

//     fn push_to_pinned_vec<P: PinnedVec<Self::O>>(&self, i: Self::I, vector: &mut P) -> Self::Res {
//         todo!()
//     }

//     fn next_any(&self, i: Self::I) -> Self::ResOf<Self::O> {
//         todo!()
//     }

//     fn next(&self, i: Self::I) -> Self::ResOf<Self::O> {
//         todo!()
//     }

//     fn reduce<R>(&self, reduce: R, i: Self::I, acc: Option<Self::O>) -> Self::ResOf<Option<Self::O>>
//     where
//         R: Fn(Self::O, Self::O) -> Self::O,
//     {
//         todo!()
//     }
// }
