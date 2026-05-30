use crate::infallible_use::Using;
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::PinnedVec;

pub struct UseBagDepr<U>
where
    U: Using,
    U::Item: Send,
{
    using: U,
    cache: ConcurrentBag<(usize, U::Item)>,
}

impl<U> UseBagDepr<U>
where
    U: Using,
    U::Item: Send,
{
    pub fn new(using: U) -> Self {
        let cache = ConcurrentBag::new();
        Self { using, cache }
    }

    pub fn into_vec(self) -> Vec<U::Item> {
        let mut vec = self.cache.into_inner();
        vec.sort_by_key(|(th_idx, _)| *th_idx);
        vec.into_iter().map(|(_, value)| value).collect()
    }
}

impl<U> Using for UseBagDepr<U>
where
    U: Using,
    U::Item: Send,
{
    type Item = U::Item;

    type ItemBorrow<'a>
        = &'a mut U::Item
    where
        Self: 'a;

    fn create(&self, thread_idx: usize) -> Self::Item {
        let use_var = self.using.create(thread_idx);
        // self.cache.push((thread_idx, use_var));
        todo!()
    }

    #[inline]
    fn get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        let use_var = self.using.create(thread_idx);
        let idx = self.cache.push((thread_idx, use_var));
        // SAFETY: it is safe to access to the index as it is
        // pushed / initialized just above. Further, `get` will
        // be called exactly once by the corresponding thread,
        // and hence, there will be no race condition.
        let a = unsafe { &mut *self.cache.ptr_mut(idx) };
        &mut a.1
    }

    fn get_mut(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        todo!()
    }
}
