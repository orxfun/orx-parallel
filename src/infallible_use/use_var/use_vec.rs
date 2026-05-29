use crate::infallible_use::Use;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;

pub struct UseVec<U>
where
    U: Use,
    U::Item: Send,
{
    using: U,
    cache: ConcurrentBag<(usize, U::Item)>,
}

impl<U> Use for UseVec<U>
where
    U: Use,
    U::Item: Send,
{
    type Item = U::Item;

    type ItemKind = U::Item;

    fn create(&self, thread_idx: usize) -> Self::Item {
        let use_var = self.using.create(thread_idx);
        // self.cache.push((thread_idx, use_var));
        todo!()
    }

    #[inline]
    fn get(&self, thread_idx: usize) -> Self::ItemKind {
        let use_var = self.using.create(thread_idx);
        let position = self.cache.push((thread_idx, use_var));
        // unsafe {self.cache.};
        todo!()
    }
}
