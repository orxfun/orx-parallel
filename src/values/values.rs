use crate::values::{
    WhilstOption,
    runner_results::{
        ArbitraryPush, Fallibility, Next, OrderedPush, Reduce, SequentialPush, Stop, StopReduce,
        StopWithIdx,
    },
};
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub trait Values: Sized {
    type Item;

    type Fallibility: Fallibility;

    fn values_to_depracate(self) -> impl IntoIterator<Item = Self::Item>;

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>;

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility>;

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send;

    /// Returns (true, _) if the computation must early exit.
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item;

    fn u_acc_reduce<U, X>(
        self,
        u: &mut U,
        acc: Option<Self::Item>,
        reduce: X,
    ) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item;

    fn first_to_depracate(self) -> WhilstOption<Self::Item>;

    fn next(self) -> Next<Self>;

    // provided

    #[inline(always)]
    fn ordered_push_to_stop(
        ordered_push: OrderedPush<Self::Fallibility>,
    ) -> Option<StopWithIdx<<Self::Fallibility as Fallibility>::Error>> {
        <Self::Fallibility as Fallibility>::ordered_push_to_stop(ordered_push)
    }

    #[inline(always)]
    fn arbitrary_push_to_stop(
        arbitrary_push: ArbitraryPush<Self::Fallibility>,
    ) -> Option<Stop<<Self::Fallibility as Fallibility>::Error>> {
        <Self::Fallibility as Fallibility>::arbitrary_push_to_stop(arbitrary_push)
    }

    #[inline(always)]
    fn sequential_push_to_stop(
        sequential_push: SequentialPush<Self::Fallibility>,
    ) -> Option<Stop<<Self::Fallibility as Fallibility>::Error>> {
        <Self::Fallibility as Fallibility>::sequential_push_to_stop(sequential_push)
    }

    #[inline(always)]
    fn reduce_to_stop(reduce: Reduce<Self>) -> Result<Option<Self::Item>, StopReduce<Self>> {
        <Self::Fallibility as Fallibility>::reduce_to_stop(reduce)
    }
}
