use crate::generic_values::{
    TransformableValues, Vector, WhilstOption, option_result::OptionResult,
};

impl<T> TransformableValues for Option<T> {
    type Map<M, O>
        = Option<O>
    where
        M: Fn(Self::Item) -> O;
    #[inline(always)]
    fn map<M, O>(self, map: M) -> Self::Map<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        self.map(map)
    }

    type Filter<F>
        = Option<T>
    where
        F: Fn(&Self::Item) -> bool;
    #[inline(always)]
    fn filter<F>(self, filter: F) -> Self::Filter<F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        self.filter(filter)
    }

    type FlatMap<Fm, Vo>
        = Vector<core::iter::FlatMap<core::option::IntoIter<T>, Vo, Fm>>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;
    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMap<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.into_iter().flat_map(flat_map))
    }

    type FilterMap<Fm, O>
        = Option<O>
    where
        Fm: Fn(Self::Item) -> Option<O>;
    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMap<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        match self {
            Some(x) => filter_map(x),
            _ => None,
        }
    }

    type Whilst<W>
        = WhilstOption<T>
    where
        W: Fn(&Self::Item) -> bool;
    fn whilst<W>(self, whilst: W) -> Self::Whilst<W>
    where
        W: Fn(&Self::Item) -> bool,
    {
        match self {
            Some(x) => match whilst(&x) {
                true => WhilstOption::ContinueSome(x),
                false => WhilstOption::Stop,
            },
            _ => WhilstOption::ContinueNone,
        }
    }

    type MapWhileOk<Mr, O, E>
        = OptionResult<O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send;
    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> Self::MapWhileOk<Mr, O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send,
    {
        let opt_res = self.map(map_res);
        OptionResult(opt_res)
    }

    type UMap<U, M, O>
        = Option<O>
    where
        M: Fn(*mut U, Self::Item) -> O;
    #[inline(always)]
    fn u_map<U, M, O>(self, u: *mut U, map: M) -> Self::UMap<U, M, O>
    where
        M: Fn(*mut U, Self::Item) -> O,
    {
        self.map(|x| map(u, x))
    }

    type UFilter<U, F>
        = Option<T>
    where
        F: Fn(*mut U, &Self::Item) -> bool;
    #[inline(always)]
    fn u_filter<U, F>(self, u: *mut U, filter: F) -> Self::UFilter<U, F>
    where
        F: Fn(*mut U, &Self::Item) -> bool,
    {
        self.filter(|x| filter(u, x))
    }

    type UFlatMap<U, Fm, Vo>
        = Vector<core::iter::Flatten<core::option::IntoIter<Vo>>>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo;
    fn u_flat_map<U, Fm, Vo>(self, u: *mut U, flat_map: Fm) -> Self::UFlatMap<U, Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo,
    {
        let iter = self.map(|x| flat_map(u, x)).into_iter().flatten();
        Vector(iter)
    }

    type UFilterMap<U, Fm, O>
        = Option<O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>;
    fn u_filter_map<U, Fm, O>(self, u: *mut U, filter_map: Fm) -> Self::UFilterMap<U, Fm, O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>,
    {
        match self {
            Some(x) => filter_map(u, x),
            _ => None,
        }
    }
}
