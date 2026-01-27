use core::marker::PhantomData;

pub struct SliceIter<'a, T: 'a> {
    data: *const T,
    exclusive_end: *const T,
    phantom: PhantomData<&'a ()>,
}

impl<T> Default for SliceIter<'_, T> {
    fn default() -> Self {
        Self {
            data: core::ptr::null(),
            exclusive_end: core::ptr::null(),
            phantom: PhantomData,
        }
    }
}

impl<T> SliceIter<'_, T> {
    pub fn new(data: *const T, len: usize) -> Self {
        match len {
            0 => Self::default(),
            _ => Self {
                data,
                exclusive_end: unsafe { data.add(len) },
                phantom: PhantomData,
            },
        }
    }
}

impl<'a, T: 'a> Iterator for SliceIter<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self.data != self.exclusive_end {
            true => {
                let value = unsafe { &*self.data };
                self.data = unsafe { self.data.add(1) };
                Some(value)
            }
            false => None,
        }
    }
}
