use crate::collectables::ParExtendCore;
use crate::collectables::par_extend_impl::utils::{ColAndPos, IdxLen};
use alloc::vec::Vec;

pub struct Soa2<T1, T2> {
    v1: Vec<T1>,
    v2: Vec<T2>,
}

impl<T1, T2> Soa2<T1, T2> {
    pub fn new() -> Self {
        Self {
            v1: Vec::new(),
            v2: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            v1: Vec::with_capacity(capacity),
            v2: Vec::with_capacity(capacity),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, (i1, i2): (T1, T2)) {
        self.v1.push(i1);
        self.v2.push(i2);
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.v1.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.v1.is_empty()
    }
}

impl<T1, T2> Default for Soa2<T1, T2> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T1, T2> Extend<(T1, T2)> for Soa2<T1, T2> {
    fn extend<I: IntoIterator<Item = (T1, T2)>>(&mut self, iter: I) {
        for (i1, i2) in iter {
            self.v1.push(i1);
            self.v2.push(i2);
        }
    }
}

impl<T1: Send, T2: Send> ParExtendCore<(T1, T2)> for Soa2<T1, T2> {
    type ThreadValues = Self;

    type OrderedThreadValues = ColAndPos<Self>;

    fn new_thread_values() -> Self::ThreadValues {
        Self::ThreadValues::new()
    }

    fn new_ordered_thread_values() -> Self::OrderedThreadValues {
        Default::default()
    }

    fn add_thread_value(collected: &mut Self::ThreadValues, value: (T1, T2)) {
        collected.push(value);
    }

    fn add_thread_values(
        collected: &mut Self::ThreadValues,
        values: impl IntoIterator<Item = (T1, T2)>,
    ) {
        collected.extend(values)
    }

    fn add_ordered_thread_value(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        value: (T1, T2),
    ) {
        collected.values.push(value);
        collected.positions.push(IdxLen { idx, len: 1 });
    }

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = (T1, T2)>,
    ) {
        let len_begin = collected.values.len();
        collected.values.extend(values);

        let len = collected.values.len() - len_begin;
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
        }
    }

    fn add_ordered_thread_optionals(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Option<(T1, T2)>>,
    ) -> Option<()> {
        todo!()
    }

    fn add_ordered_thread_fallibles<E>(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Result<(T1, T2), E>>,
    ) -> Result<(), E> {
        todo!()
    }

    fn add_one(&mut self, value: (T1, T2)) {
        todo!()
    }

    fn extend_merge_infallibles(&mut self, thread_results: Vec<Self::ThreadValues>) {
        todo!()
    }

    fn extend_merge_ordered_infallibles(&mut self, thread_results: Vec<Self::OrderedThreadValues>) {
        todo!()
    }
}
