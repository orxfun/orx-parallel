// use super::par_collect_into::ParCollectIntoCore;
// use crate::Params;
// use crate::collect_into::utils::split_vec_reserve;
// use crate::computational_variants::computations::{
//     map_collect_into, xap_collect_into, xap_try_collect_into,
// };
// use crate::generic_values::runner_results::{Fallibility, Infallible};
// use crate::generic_values::{TransformableValues, Values};
// use crate::orch::Orchestrator;
// use orx_concurrent_iter::ConcurrentIter;
// #[cfg(test)]
// use orx_pinned_vec::PinnedVec;
// use orx_split_vec::{GrowthWithConstantTimeAccess, PseudoDefault, SplitVec};

// impl<O, G> ParCollectIntoCore<O> for SplitVec<O, G>
// where
//     O: Send + Sync,
//     G: GrowthWithConstantTimeAccess,
//     Self: PseudoDefault,
// {
//     type BridgePinnedVec = Self;

//     fn empty(iter_len: Option<usize>) -> Self {
//         let mut vec = Self::pseudo_default();
//         split_vec_reserve(&mut vec, false, iter_len);
//         vec
//     }

//     fn m_collect_into<R, I, M1>(
//         mut self,
//         orchestrator: R,
//         params: Params,
//         iter: I,
//         map1: M1,
//     ) -> Self
//     where
//         R: Orchestrator,
//         I: ConcurrentIter,
//         M1: Fn(I::Item) -> O + Sync,
//         O: Send,
//     {
//         split_vec_reserve(&mut self, params.is_sequential(), iter.try_get_len());
//         let (_, pinned_vec) = map_collect_into(orchestrator, params, iter, map1, self);
//         pinned_vec
//     }

//     fn x_collect_into<R, I, Vo, X1>(
//         mut self,
//         orchestrator: R,
//         params: Params,
//         iter: I,
//         xap1: X1,
//     ) -> Self
//     where
//         R: Orchestrator,
//         I: ConcurrentIter,
//         Vo: TransformableValues<Item = O, Fallibility = Infallible>,
//         X1: Fn(I::Item) -> Vo + Sync,
//     {
//         split_vec_reserve(&mut self, params.is_sequential(), iter.try_get_len());
//         let (_num_spawned, pinned_vec) = xap_collect_into(orchestrator, params, iter, xap1, self);
//         pinned_vec
//     }

//     fn x_try_collect_into<R, I, Vo, X1>(
//         mut self,
//         orchestrator: R,
//         params: Params,
//         iter: I,
//         xap1: X1,
//     ) -> Result<Self, <Vo::Fallibility as Fallibility>::Error>
//     where
//         R: Orchestrator,
//         I: ConcurrentIter,
//         X1: Fn(I::Item) -> Vo + Sync,
//         Vo: Values<Item = O>,
//         Self: Sized,
//     {
//         split_vec_reserve(&mut self, params.is_sequential(), iter.try_get_len());
//         let (_num_spawned, result) = xap_try_collect_into(orchestrator, params, iter, xap1, self);
//         result
//     }

//     // test

//     #[cfg(test)]
//     fn length(&self) -> usize {
//         self.len()
//     }
// }
