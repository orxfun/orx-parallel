use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

use crate::computations::{Values, Vector};

pub struct Continue<T>(Option<T>);

impl<T> IntoIterator for Continue<T> {
    type Item = T;

    type IntoIter = <Option<T> as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Values for Continue<T> {
    type Item = T;

    type Mapped<M, O>
        = Continue<O>
    where
        M: Fn(Self::Item) -> O;

    type Filtered<F>
        = Continue<T>
    where
        F: Fn(&Self::Item) -> bool;

    type FlatMapped<Fm, Vo>
        = Vector<core::iter::FlatMap<<Continue<T> as IntoIterator>::IntoIter, Vo, Fm>>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    type FilterMapped<Fm, O>
        = Continue<O>
    where
        Fm: Fn(Self::Item) -> Option<O>;

    #[inline(always)]
    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        self
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: orx_fixed_vec::PinnedVec<Self::Item>,
    {
        if let Some(x) = self.0 {
            vector.push(x)
        }
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        if let Some(x) = self.0 {
            bag.push(x);
        }
    }

    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        if let Some(x) = self.0 {
            unsafe { o_bag.set_value(idx, x) };
        }
    }

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) {
        if let Some(x) = self.0 {
            vec.push((idx, x))
        }
    }

    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        Continue(self.0.map(map))
    }

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.into_iter().flat_map(flat_map))
    }

    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        Continue(match self.0 {
            Some(x) => filter_map(x),
            _ => None,
        })
    }

    fn filter<F>(self, filter: F) -> Self::Filtered<F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        Continue(match self.0 {
            Some(x) if filter(&x) => Some(x),
            _ => None,
        })
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match (acc, self.0) {
            (Some(x), Some(y)) => Some(reduce(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match (acc, self.0) {
            (Some(x), Some(y)) => Some(reduce(u, x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

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
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item,
    {
        match self.0 {
            Some(x) if filter(&x) => {
                let vo = map2(x);
                vo.acc_reduce(acc, reduce)
            }
            _ => acc,
        }
    }

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
        X: Fn(&mut U, Vo::Item, Vo::Item) -> Vo::Item,
    {
        match self.0 {
            Some(x) if filter(u, &x) => {
                let vo = map2(u, x);
                vo.u_acc_reduce(u, acc, reduce)
            }
            _ => acc,
        }
    }

    fn first(self) -> Option<Self::Item> {
        self.0
    }

    fn fx_next<F, M2, Vo>(self, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
    {
        match self.0 {
            Some(x) if filter(&x) => map2(x).first(),
            _ => None,
        }
    }

    fn u_fx_next<U, F, M2, Vo>(self, u: &mut U, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
        Vo: Values,
    {
        match self.0 {
            Some(x) if filter(u, &x) => map2(u, x).first(),
            _ => None,
        }
    }

    fn filter_map_collect_sequential<F, M2, P, Vo>(self, filter: F, map2: M2, vector: &mut P)
    where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        match self.0 {
            Some(x) if filter(&x) => {
                let vo = map2(x);
                vo.push_to_pinned_vec(vector);
            }
            _ => {}
        }
    }

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
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        match self.0 {
            Some(x) if filter(u, &x) => {
                let vo = map2(u, x);
                vo.push_to_pinned_vec(vector);
            }
            _ => {}
        }
    }

    fn filter_map_collect_arbitrary<F, M2, P, Vo>(
        self,
        filter: F,
        map2: M2,
        bag: &orx_concurrent_bag::ConcurrentBag<Vo::Item, P>,
    ) where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
        Vo::Item: Send,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        match self.0 {
            Some(x) if filter(&x) => {
                let vo = map2(x);
                vo.push_to_bag(bag);
            }
            _ => {}
        }
    }

    fn u_filter_map_collect_arbitrary<U, F, M2, P, Vo>(
        self,
        u: &mut U,
        filter: F,
        map2: M2,
        bag: &orx_concurrent_bag::ConcurrentBag<Vo::Item, P>,
    ) where
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>,
        Vo::Item: Send,
    {
        match self.0 {
            Some(x) if filter(u, &x) => {
                let vo = map2(u, x);
                vo.push_to_bag(bag);
            }
            _ => {}
        }
    }
}
