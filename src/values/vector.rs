use super::transformable_values::TransformableValues;
use crate::values::{
    Values, VectorResult, WhilstAtom, WhilstOption, WhilstVector,
    runner_results::{
        ArbitraryPush, Fallible, Infallible, Next, OrderedPush, Reduce, SequentialPush,
    },
};
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub struct Vector<I>(pub I)
where
    I: IntoIterator;

impl<I> Values for Vector<I>
where
    I: IntoIterator,
{
    type Item = I::Item;

    type Fallibility = Infallible;

    fn values_to_depracate(self) -> impl IntoIterator<Item = Self::Item> {
        self.0
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            vector.push(x);
        }
        SequentialPush::Done
    }

    #[inline(always)]
    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        for x in self.0 {
            vec.push((idx, x));
        }
        OrderedPush::Done
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        for x in self.0 {
            bag.push(x);
        }
        ArbitraryPush::Done
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        let reduced = self.0.into_iter().reduce(&reduce);

        Reduce::Done {
            acc: match (acc, reduced) {
                (Some(x), Some(y)) => Some(reduce(x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        let reduced = self.0.into_iter().reduce(|a, b| reduce(u, a, b));

        Reduce::Done {
            acc: match (acc, reduced) {
                (Some(x), Some(y)) => Some(reduce(u, x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        }
    }

    #[inline(always)]
    fn first_to_depracate(self) -> WhilstOption<Self::Item> {
        match self.0.into_iter().next() {
            Some(x) => WhilstOption::ContinueSome(x),
            None => WhilstOption::ContinueNone,
        }
    }

    fn next(self) -> Next<Self> {
        Next::Done {
            value: self.0.into_iter().next(),
        }
    }
}

impl<I> TransformableValues for Vector<I>
where
    I: IntoIterator,
{
    #[inline(always)]
    fn map<M, O>(
        self,
        map: M,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(Self::Item) -> O,
    {
        Vector(self.0.into_iter().map(map))
    }

    #[inline(always)]
    fn filter<F>(
        self,
        filter: F,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        F: Fn(&Self::Item) -> bool,
    {
        Vector(self.0.into_iter().filter(filter))
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(
        self,
        flat_map: Fm,
    ) -> impl TransformableValues<Item = Vo::Item, Fallibility = Self::Fallibility>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.0.into_iter().flat_map(flat_map))
    }

    #[inline(always)]
    fn filter_map<Fm, O>(
        self,
        filter_map: Fm,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        Vector(self.0.into_iter().filter_map(filter_map))
    }

    fn whilst(
        self,
        whilst: impl Fn(&Self::Item) -> bool,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        Self: Sized,
    {
        let iter = self.0.into_iter().map(move |x| match whilst(&x) {
            true => WhilstAtom::Continue(x),
            false => WhilstAtom::Stop,
        });
        WhilstVector(iter)
    }

    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> impl Values<Item = O, Fallibility = Fallible<E>>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send,
    {
        let iter_res = self.0.into_iter().map(move |x| map_res(x));
        VectorResult(iter_res)
    }

    fn u_map<U, M, O>(
        self,
        u: &mut U,
        map: M,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(&mut U, Self::Item) -> O,
    {
        Vector(self.0.into_iter().map(move |x| map(u, x)))
    }
}
