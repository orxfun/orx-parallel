use crate::collectables::ParExtendCore;
use crate::collectables::par_extend_impl::utils::{ColAndPos, IdxLen};
use crate::collectables::par_extend_impl::vec::ThBegLen;
use alloc::{vec, vec::Vec};
use core::iter::Zip;
use orx_priority_queue::{BinaryHeap, PriorityQueue};

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

    pub fn into_inner(self) -> (Vec<T1>, Vec<T2>) {
        (self.v1, self.v2)
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

    pub fn reserve(&mut self, additional: usize) {
        self.v1.reserve(additional);
        self.v2.reserve(additional);
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        unsafe { self.v1.set_len(new_len) };
        unsafe { self.v2.set_len(new_len) };
    }

    pub fn as_ptr(&self) -> (*const T1, *const T2) {
        (self.v1.as_ptr(), self.v2.as_ptr())
    }

    pub fn as_mut_ptr(&mut self) -> (*mut T1, *mut T2) {
        (self.v1.as_mut_ptr(), self.v2.as_mut_ptr())
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

impl<T1, T2> IntoIterator for Soa2<T1, T2> {
    type Item = (T1, T2);

    type IntoIter = Zip<vec::IntoIter<T1>, vec::IntoIter<T2>>;

    fn into_iter(self) -> Self::IntoIter {
        self.v1.into_iter().zip(self.v2)
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

    // opt: thread collect

    fn add_ordered_thread_optionals(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Option<(T1, T2)>>,
    ) -> Option<()> {
        let len_begin = collected.values.len();
        for value in values {
            collected.values.push(value?);
        }

        let len = collected.values.len() - len_begin;
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
        }

        Some(())
    }

    // res: thread collect

    fn add_ordered_thread_fallibles<E>(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Result<(T1, T2), E>>,
    ) -> Result<(), E> {
        let len_begin = collected.values.len();
        for value in values {
            collected.values.push(value?);
        }

        let len = collected.values.len() - len_begin;
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
        }

        Ok(())
    }

    // add

    #[inline(always)]
    fn add_one(&mut self, value: (T1, T2)) {
        self.push(value);
    }

    // extend - merge

    fn extend_merge_infallibles(&mut self, results: Vec<Self::ThreadValues>) {
        let collected_len: usize = results.iter().map(|x| x.len()).sum();
        self.reserve(collected_len);
        for result in results {
            self.extend(result);
        }
    }

    fn extend_merge_ordered_infallibles(&mut self, mut results: Vec<Self::OrderedThreadValues>) {
        let collected_len: usize = results.iter().map(|x| x.values.len()).sum();
        self.reserve(collected_len);
        let initial_len = self.len();
        let total_len = initial_len + collected_len;

        let mut queue = BinaryHeap::with_capacity(results.len());
        let mut pos_indices = vec![0; results.len()];

        for (t, vec) in results.iter().enumerate() {
            if let Some(pos) = vec.positions.first() {
                let node = ThBegLen::new(t, 0, pos.len);
                queue.push(node, pos.idx);
            }
        }
        let mut curr_t = queue.pop_node();
        let mut ptr_dst = unsafe { mut_ptr_add(self.as_mut_ptr(), initial_len) };

        while let Some(ThBegLen { th, beg, len }) = curr_t {
            let ptr_src = unsafe { ptr_add(results[th].values.as_ptr(), beg) };
            unsafe { copy_nonoverlapping(ptr_src, ptr_dst, len) };

            pos_indices[th] += 1;
            curr_t = match results[th].positions.get(pos_indices[th]) {
                Some(pos) => {
                    let beg = beg + len;
                    let node = ThBegLen::new(th, beg, pos.len);
                    Some(queue.push_then_pop(node, pos.idx).0)
                }
                None => queue.pop_node(),
            };

            ptr_dst = unsafe { mut_ptr_add(ptr_dst, len) };
        }

        for vec in results.iter_mut() {
            // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
            // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
            unsafe { vec.values.set_len(0) };
        }

        unsafe { self.set_len(total_len) };
    }
}

unsafe fn ptr_add<T1, T2>((a, b): (*const T1, *const T2), count: usize) -> (*const T1, *const T2) {
    (unsafe { a.add(count) }, unsafe { b.add(count) })
}

unsafe fn mut_ptr_add<T1, T2>((a, b): (*mut T1, *mut T2), count: usize) -> (*mut T1, *mut T2) {
    (unsafe { a.add(count) }, unsafe { b.add(count) })
}

unsafe fn copy_nonoverlapping<T1, T2>(
    ptr_src: (*const T1, *const T2),
    ptr_dst: (*mut T1, *mut T2),
    len: usize,
) {
    unsafe { ptr_dst.0.copy_from_nonoverlapping(ptr_src.0, len) };
    unsafe { ptr_dst.1.copy_from_nonoverlapping(ptr_src.1, len) };
}
