use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub trait Values {
    type Item;

    fn values(self) -> impl IntoIterator<Item = Self::Item>;

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> Option<usize>
    where
        P: PinnedVec<Self::Item>;

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize>;

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> Option<usize>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send;

    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O;

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>;

    fn acc_reduce<X>(
        self,
        acc: Option<Self::Item>,
        reduce: X,
    ) -> (Option<usize>, Option<Self::Item>)
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

    fn first(self) -> Option<Self::Item>;
}
