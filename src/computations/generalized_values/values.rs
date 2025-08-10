use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub trait Values {
    type Item;

    type Mapped<M, O>: Values<Item = O>
    where
        M: Fn(Self::Item) -> O;

    type Filtered<F>: Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool;

    type FlatMapped<Fm, Vo>: Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    type FilterMapped<Fm, O>: Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>;

    fn values(self) -> impl IntoIterator<Item = Self::Item>;

    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: PinnedVec<Self::Item>;

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send;

    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send;

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>);

    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        M: Fn(Self::Item) -> O;

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>;

    fn filter<F>(self, filter: F) -> Self::Filtered<F>
    where
        F: Fn(&Self::Item) -> bool;

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
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

    fn fx_reduce<F, M2, Vo, X>(
        self,
        acc: Option<Vo::Item>,
        filter: F,
        map2: M2,
        reduce: X,
    ) -> Option<Vo::Item>
    where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item;

    fn u_fx_reduce<U, F, M2, Vo, X>(
        self,
        u: &mut U,
        acc: Option<Vo::Item>,
        filter: F,
        map2: M2,
        reduce: X,
    ) -> Option<Vo::Item>
    where
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
        Vo: Values,
        X: Fn(&mut U, Vo::Item, Vo::Item) -> Vo::Item;

    fn first(self) -> Option<Self::Item>;

    fn fx_next<F, M2, Vo>(self, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values;

    fn u_fx_next<U, F, M2, Vo>(self, u: &mut U, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
        Vo: Values;

    fn filter_map_collect_sequential<F, M2, P, Vo>(self, filter: F, map2: M2, vector: &mut P)
    where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>;

    fn u_filter_map_collect_sequential<U, F, M2, P, Vo>(
        self,
        u: &mut U,
        filter: F,
        map2: M2,
        vector: &mut P,
    ) where
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>;
}
