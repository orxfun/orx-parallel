use crate::generic_values::{
    TransformableValues, WhilstOption, WhilstVector, transformable::iter::WhilstOptionIter,
    whilst_option_result::WhilstOptionResult,
};

impl<T> TransformableValues for WhilstOption<T> {
    type Map<M, O>
        = WhilstOption<O>
    where
        M: Fn(Self::Item) -> O;
    fn map<M, O>(self, map: M) -> Self::Map<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        match self {
            Self::ContinueSome(x) => WhilstOption::ContinueSome(map(x)),
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
        }
    }

    type Inspect<F>
        = Self
    where
        F: Fn(&Self::Item);
    fn inspect<F, O>(self, inspect: F) -> Self::Inspect<F>
    where
        F: Fn(&Self::Item),
    {
        match self {
            Self::ContinueSome(x) => {
                inspect(&x);
                Self::ContinueSome(x)
            }
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
        }
    }

    type Filter<F>
        = WhilstOption<T>
    where
        F: Fn(&Self::Item) -> bool;
    fn filter<F>(self, filter: F) -> Self::Filter<F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        match self {
            Self::ContinueSome(x) => match filter(&x) {
                true => Self::ContinueSome(x),
                false => Self::ContinueNone,
            },
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
        }
    }

    type FlatMap<Fm, Vo>
        = WhilstVector<WhilstOptionIter<Vo::IntoIter>, Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMap<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        let iter = match self {
            Self::ContinueSome(x) => WhilstOption::ContinueSome(flat_map(x).into_iter()),
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
        };
        let iter = WhilstOptionIter::new(iter);
        WhilstVector(iter)
    }

    type FilterMap<Fm, O>
        = WhilstOption<O>
    where
        Fm: Fn(Self::Item) -> Option<O>;
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMap<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        match self {
            Self::ContinueSome(x) => match filter_map(x) {
                Some(x) => WhilstOption::ContinueSome(x),
                None => WhilstOption::ContinueNone,
            },
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
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
            Self::ContinueSome(x) => match whilst(&x) {
                true => Self::ContinueSome(x),
                false => Self::Stop,
            },
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
        }
    }

    type MapWhileOk<Mr, O, E>
        = WhilstOptionResult<O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send;
    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> Self::MapWhileOk<Mr, O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send,
    {
        match self {
            Self::ContinueSome(x) => match map_res(x) {
                Ok(x) => WhilstOptionResult::ContinueSomeOk(x),
                Err(e) => WhilstOptionResult::StopErr(e),
            },
            Self::ContinueNone => WhilstOptionResult::ContinueNone,
            Self::Stop => WhilstOptionResult::StopWhile,
        }
    }

    type UMap<U, M, O>
        = WhilstOption<O>
    where
        M: Fn(*mut U, Self::Item) -> O;
    fn u_map<U, M, O>(self, u: *mut U, map: M) -> Self::UMap<U, M, O>
    where
        M: Fn(*mut U, Self::Item) -> O,
    {
        match self {
            Self::ContinueSome(x) => WhilstOption::ContinueSome(map(u, x)),
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
        }
    }

    type UFilter<U, F>
        = WhilstOption<T>
    where
        F: Fn(*mut U, &Self::Item) -> bool;
    fn u_filter<U, F>(self, u: *mut U, filter: F) -> Self::UFilter<U, F>
    where
        F: Fn(*mut U, &Self::Item) -> bool,
    {
        match self {
            Self::ContinueSome(x) => match filter(u, &x) {
                true => Self::ContinueSome(x),
                false => Self::ContinueNone,
            },
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
        }
    }

    type UFlatMap<U, Fm, Vo>
        = WhilstVector<WhilstOptionIter<Vo::IntoIter>, Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo;
    fn u_flat_map<U, Fm, Vo>(self, u: *mut U, flat_map: Fm) -> Self::UFlatMap<U, Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo,
    {
        let iter = match self {
            Self::ContinueSome(x) => WhilstOption::ContinueSome(flat_map(u, x).into_iter()),
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
        };
        let iter = WhilstOptionIter::new(iter);
        WhilstVector(iter)
    }

    type UFilterMap<U, Fm, O>
        = WhilstOption<O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>;
    fn u_filter_map<U, Fm, O>(self, u: *mut U, filter_map: Fm) -> Self::UFilterMap<U, Fm, O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>,
    {
        match self {
            Self::ContinueSome(x) => match filter_map(u, x) {
                Some(x) => WhilstOption::ContinueSome(x),
                None => WhilstOption::ContinueNone,
            },
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
        }
    }
}
