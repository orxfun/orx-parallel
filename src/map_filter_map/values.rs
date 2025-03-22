use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub trait Values {
    type Item;

    type Mapped<M, O>: Values<Item = O>
    where
        M: Fn(Self::Item) -> O;

    type FlatMapped<Fm, Vo>: Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    fn values(self) -> impl IntoIterator<Item = Self::Item>;

    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: PinnedVec<Self::Item>;

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync;

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>);

    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        M: Fn(Self::Item) -> O;

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    fn filter_map_collect_sequential<F, M2, P, Vo, O>(self, filter: F, map2: M2, vector: &mut P)
    where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>;

    fn filter_map_collect_arbitrary<F, M2, P, Vo, O>(
        self,
        filter: F,
        map2: M2,
        bag: &ConcurrentBag<O, P>,
    ) where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
        O: Send + Sync;

    fn filter_map_collect_heap<F, M2, Vo, O>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        vec: &mut Vec<(usize, O)>,
    ) where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        O: Send + Sync;
}

pub struct Atom<T>(pub T);

impl<T> Values for Atom<T> {
    type Item = T;

    type Mapped<M, O>
        = Atom<O>
    where
        M: Fn(Self::Item) -> O;

    type FlatMapped<Fm, Vo>
        = Vector<Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    fn values(self) -> impl IntoIterator<Item = T> {
        core::iter::once(self.0)
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: PinnedVec<T>,
    {
        vector.push(self.0);
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<T, P>)
    where
        P: IntoConcurrentPinnedVec<T>,
        T: Send + Sync,
    {
        bag.push(self.0);
    }

    #[inline(always)]
    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, T)>) {
        vec.push((idx, self.0))
    }

    #[inline(always)]
    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        Atom(map(self.0))
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(flat_map(self.0))
    }

    #[inline(always)]
    fn filter_map_collect_sequential<F, M2, P, Vo, O>(self, filter: F, map2: M2, vector: &mut P)
    where
        Self: Sized,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_pinned_vec(vector);
        }
    }

    #[inline(always)]
    fn filter_map_collect_arbitrary<F, M2, P, Vo, O>(
        self,
        filter: F,
        map2: M2,
        bag: &ConcurrentBag<O, P>,
    ) where
        Self: Sized,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
        O: Send + Sync,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_bag(bag);
        }
    }

    #[inline(always)]
    fn filter_map_collect_heap<F, M2, Vo, O>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        vec: &mut Vec<(usize, O)>,
    ) where
        Self: Sized,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        O: Send + Sync,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_vec_with_idx(input_idx, vec);
        }
    }
}

pub struct Vector<I>(pub I)
where
    I: IntoIterator;

impl<I> Values for Vector<I>
where
    I: IntoIterator,
{
    type Item = I::Item;

    type Mapped<M, O>
        = Vector<core::iter::Map<I::IntoIter, M>>
    where
        M: Fn(Self::Item) -> O;

    type FlatMapped<Fm, Vo>
        = Vector<core::iter::FlatMap<I::IntoIter, Vo, Fm>>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        self.0
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            vector.push(x);
        }
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync,
    {
        for x in self.0 {
            bag.push(x);
        }
    }

    #[inline(always)]
    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) {
        for x in self.0 {
            vec.push((idx, x))
        }
    }

    #[inline(always)]
    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        Vector(self.0.into_iter().map(map))
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.0.into_iter().flat_map(flat_map))
    }

    #[inline]
    fn filter_map_collect_sequential<F, M2, P, Vo, O>(self, filter: F, map2: M2, vector: &mut P)
    where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
    {
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_pinned_vec(vector);
            }
        }
    }

    #[inline]
    fn filter_map_collect_arbitrary<F, M2, P, Vo, O>(
        self,
        filter: F,
        map2: M2,
        bag: &ConcurrentBag<O, P>,
    ) where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
        O: Send + Sync,
    {
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_bag(bag);
            }
        }
    }

    #[inline]
    fn filter_map_collect_heap<F, M2, Vo, O>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        vec: &mut Vec<(usize, O)>,
    ) where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        O: Send + Sync,
    {
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_vec_with_idx(input_idx, vec);
            }
        }
    }
}
