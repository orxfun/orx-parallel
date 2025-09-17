use crate::Params;
use crate::generic_values::runner_results::{
    Fallibility, Infallible, ParallelCollect, ParallelCollectArbitrary, Stop,
};
use crate::orch::{NumSpawned, Orchestrator};
use crate::runner::parallel_runner_compute as prc;
use crate::using::using_variants::Using;
use crate::{IterationOrder, generic_values::Values};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// pub fn map_collect_into<U, R, I, O, M1, P>(
//     using: U,
//     orchestrator: R,
//     params: Params,
//     iter: I,
//     map1: M1,
//     pinned_vec: P,
// ) -> (NumSpawned, P)
// where
//     U: Using,
//     R: Orchestrator,
//     I: ConcurrentIter,
//     M1: Fn(I::Item) -> O + Sync,
//     O: Send,
//     P: IntoConcurrentPinnedVec<O>,
// {
//     match (params.is_sequential(), params.iteration_order) {
//         (true, _) => (
//             NumSpawned::zero(),
//             map_collect_into_seq(iter, map1, pinned_vec),
//         ),
//         #[cfg(test)]
//         (false, IterationOrder::Arbitrary) => {
//             prc::collect_arbitrary::m(orchestrator, params, iter, map1, pinned_vec)
//         }
//         (false, _) => prc::collect_ordered::m(orchestrator, params, iter, map1, pinned_vec),
//     }
// }

// fn map_collect_into_seq<U, I, O, M1, P>(using: U, iter: I, map1: M1, mut pinned_vec: P) -> P
// where
//     U: Using,
//     I: ConcurrentIter,
//     M1: Fn(I::Item) -> O + Sync,
//     O: Send,
//     P: IntoConcurrentPinnedVec<O>,
// {
//     let iter = iter.into_seq_iter();
//     for i in iter {
//         pinned_vec.push(map1(i));
//     }
//     pinned_vec
// }

// pub fn xap_collect_into<R, I, Vo, X1, P>(
//     orchestrator: R,
//     params: Params,
//     iter: I,
//     xap1: X1,
//     pinned_vec: P,
// ) -> (NumSpawned, P)
// where
//     R: Orchestrator,
//     I: ConcurrentIter,
//     Vo: Values<Fallibility = Infallible>,
//     Vo::Item: Send,
//     X1: Fn(I::Item) -> Vo + Sync,
//     P: IntoConcurrentPinnedVec<Vo::Item>,
// {
//     match (params.is_sequential(), params.iteration_order) {
//         (true, _) => (
//             NumSpawned::zero(),
//             xap_collect_into_seq(iter, xap1, pinned_vec),
//         ),
//         (false, IterationOrder::Arbitrary) => {
//             let (num_threads, result) =
//                 prc::collect_arbitrary::x(orchestrator, params, iter, xap1, pinned_vec);
//             let pinned_vec = match result {
//                 ParallelCollectArbitrary::AllOrUntilWhileCollected { pinned_vec } => pinned_vec,
//             };
//             (num_threads, pinned_vec)
//         }
//         (false, IterationOrder::Ordered) => {
//             let (num_threads, result) =
//                 prc::collect_ordered::x(orchestrator, params, iter, xap1, pinned_vec);
//             let pinned_vec = match result {
//                 ParallelCollect::AllCollected { pinned_vec } => pinned_vec,
//                 ParallelCollect::StoppedByWhileCondition {
//                     pinned_vec,
//                     stopped_idx: _,
//                 } => pinned_vec,
//             };
//             (num_threads, pinned_vec)
//         }
//     }
// }

// fn xap_collect_into_seq<I, Vo, X1, P>(iter: I, xap1: X1, mut pinned_vec: P) -> P
// where
//     I: ConcurrentIter,
//     Vo: Values,
//     Vo::Item: Send,
//     X1: Fn(I::Item) -> Vo + Sync,
//     P: IntoConcurrentPinnedVec<Vo::Item>,
// {
//     let iter = iter.into_seq_iter();
//     for i in iter {
//         let vt = xap1(i);
//         let done = vt.push_to_pinned_vec(&mut pinned_vec);
//         if Vo::sequential_push_to_stop(done).is_some() {
//             break;
//         }
//     }

//     pinned_vec
// }
