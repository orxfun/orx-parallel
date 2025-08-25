use super::{TransformableValues, Vector};
use crate::values::{
    Values,
    option_result::OptionResult,
    runner_results::{
        ArbitraryPush, Fallible, Infallible, Next, OrderedPush, Reduce, SequentialPush,
    },
    whilst_option::WhilstOption,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

impl<T> Values for Option<T> {
    type Item = T;

    type Fallibility = Infallible;

    #[inline(always)]
    fn values_to_depracate(self) -> impl IntoIterator<Item = Self::Item> {
        self
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        if let Some(x) = self {
            vector.push(x)
        }
        SequentialPush::Done
    }

    #[inline(always)]
    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        if let Some(x) = self {
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
        if let Some(x) = self {
            bag.push(x);
        }
        ArbitraryPush::Done
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        Reduce::Done {
            acc: match (acc, self) {
                (Some(x), Some(y)) => Some(reduce(x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        }
    }

    #[inline(always)]
    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        Reduce::Done {
            acc: match (acc, self) {
                (Some(x), Some(y)) => Some(reduce(u, x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        }
    }

    #[inline(always)]
    fn first_to_depracate(self) -> WhilstOption<Self::Item> {
        match self {
            Some(x) => WhilstOption::ContinueSome(x),
            None => WhilstOption::ContinueNone,
        }
    }

    fn next(self) -> Next<Self> {
        Next::Done { value: self }
    }
}

impl<T> TransformableValues for Option<T> {
    #[inline(always)]
    fn map<M, O>(
        self,
        map: M,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(Self::Item) -> O,
    {
        self.map(map)
    }

    #[inline(always)]
    fn filter<F>(
        self,
        filter: F,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        F: Fn(&Self::Item) -> bool,
    {
        self.filter(filter)
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
        Vector(self.into_iter().flat_map(flat_map))
    }

    #[inline(always)]
    fn filter_map<Fm, O>(
        self,
        filter_map: Fm,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        match self {
            Some(x) => filter_map(x),
            _ => None,
        }
    }

    fn whilst(
        self,
        whilst: impl Fn(&Self::Item) -> bool,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        Self: Sized,
    {
        match self {
            Some(x) => match whilst(&x) {
                true => WhilstOption::ContinueSome(x),
                false => WhilstOption::Stop,
            },
            _ => WhilstOption::ContinueNone,
        }
    }

    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> impl Values<Item = O, Fallibility = Fallible<E>>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send,
    {
        let opt_res = self.map(map_res);
        OptionResult(opt_res)
    }
}
