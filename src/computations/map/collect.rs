use super::m::M;
use crate::runner::{ComputationKind, ParallelRunner};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub struct MCollect<I, O, M1, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    m: M<I, O, M1>,
    pinned_vec: P,
}

impl<I, O, M1, P> MCollect<I, O, M1, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn sequential(self) -> P {
        let (m, mut pinned_vec) = (self.m, self.pinned_vec);
        let (_, iter, map1) = m.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(i));
        }

        pinned_vec
    }

    // fn parallel_in_input_order<R: ParallelRunner>(self) -> (usize, P) {
    //     let (m, pinned_vec) = (self.m, self.pinned_vec);
    //     let offset = pinned_vec.len();
    //     let (params, iter, map1) = m.destruct();
    //     let initial_len = iter.try_get_len();

    //     let o_bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();

    //     // let transform_i = |(i_idx, i)| unsafe { o_bag.set_value(offset + i_idx, map1(i)) };
    //     // let transform_n =
    //     //     |(i_idx, is, n)| unsafe { o_bag.set_n_values(offset + i_idx, n, is.map()) };

    //     let runner = R::new(ComputationKind::Collect, params, initial_len);
    //     let num_spawned = runner.run_with_idx(&iter, &transform_i);

    //     let values = unsafe { o_bag.into_inner().unwrap_only_if_counts_match() };
    //     (num_spawned, values)
    // }
}
