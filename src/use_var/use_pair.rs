use crate::use_var::{Use, pair_ptr::PairPtr};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_self_or::SoM;

pub struct UsePair<U, V>
where
    U: Use,
    V: Use,
{
    u: U,
    v: V,
    cache: ConcurrentOrderedBag<PairPtr<U::Item, V::Item>>,
}

impl<U, V> UsePair<U, V>
where
    U: Use,
    V: Use,
{
    pub fn new(u: U, v: V) -> Self {
        let cache = ConcurrentOrderedBag::new();
        Self { u, v, cache }
    }
}

impl<U, V> Use for UsePair<U, V>
where
    U: Use,
    V: Use,
{
    type Item = PairPtr<U::Item, V::Item>;

    type ItemBorrow<'i>
        = &'i mut PairPtr<U::Item, V::Item>
    where
        Self: 'i;

    fn init_get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        let u = self.u.init_get(thread_idx).get_mut() as *mut U::Item;
        let v = self.v.init_get(thread_idx).get_mut() as *mut V::Item;
        let pair_ptr = PairPtr::new(u, v);
        unsafe { self.cache.set_value(thread_idx, pair_ptr) };

        // SAFETY: it is safe to access to the index as it is
        // pushed / initialized just above. Further, `get` will
        // be called exactly once by the corresponding thread,
        // and hence, there will be no race condition.
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    fn get(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        assert!(self.cache.len() > thread_idx);
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    fn max_threads(&self) -> Option<usize> {
        match (self.u.max_threads(), self.v.max_threads()) {
            (Some(u), Some(v)) => Some(u.min(v)),
            (Some(u), None) => Some(u),
            (None, Some(v)) => Some(v),
            (None, None) => None,
        }
    }
}
