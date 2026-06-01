use crate::use_var::Use;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_self_or::SoM;

pub struct UseCached<'a, U: Use> {
    u: &'a mut U,
    cache: ConcurrentOrderedBag<Ptr<U::Item>>,
}

impl<'a, U: Use> UseCached<'a, U> {
    pub fn new(u: &'a mut U) -> Self {
        let cache = ConcurrentOrderedBag::new();
        Self { u, cache }
    }
}

impl<'a, U: Use> Use for UseCached<'a, U> {
    type Item = U::Item;

    type ItemBorrow<'i>
        = &'i mut U::Item
    where
        Self: 'i;

    fn init_get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        let mut use_var = self.u.init_get(thread_idx);
        let ptr = use_var.get_mut() as *mut U::Item;
        unsafe { self.cache.set_value(thread_idx, Ptr(ptr)) };

        // let idx = self.cache.push((thread_idx, use_var));
        // SAFETY: it is safe to access to the index as it is
        // pushed / initialized just above. Further, `get` will
        // be called exactly once by the corresponding thread,
        // and hence, there will be no race condition.
        unsafe { &mut *ptr }
    }

    fn get(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        assert!(self.cache.len() > 0);
        let ptr = unsafe { &*self.cache.ptr_mut(thread_idx) }.0;
        unsafe { &mut *ptr }
    }

    fn max_threads(&self) -> Option<usize> {
        self.u.max_threads()
    }
}

#[derive(Clone)]
struct Ptr<T>(*mut T);

unsafe impl<T> Send for Ptr<T> {}
