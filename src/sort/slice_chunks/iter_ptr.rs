use core::marker::PhantomData;

pub struct SliceIterPtr<'a, T: 'a> {
    data: *const T,
    exclusive_end: *const T,
    phantom: PhantomData<&'a ()>,
}

impl<T> Default for SliceIterPtr<'_, T> {
    fn default() -> Self {
        Self {
            data: core::ptr::null(),
            exclusive_end: core::ptr::null(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T> SliceIterPtr<'a, T> {
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

    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.data == self.exclusive_end
    }

    pub fn peek(&self) -> Option<*const T> {
        match !self.is_finished() {
            true => Some(self.data),
            false => None,
        }
    }

    #[inline(always)]
    pub fn peek_value(&self) -> Option<&'a T> {
        match !self.is_finished() {
            true => Some(unsafe { self.peek_value_unchecked() }),
            false => None,
        }
    }

    #[inline(always)]
    pub unsafe fn peek_value_unchecked(&self) -> &'a T {
        unsafe { &*self.data }
    }

    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> *const T {
        let value = self.data;
        self.data = unsafe { self.data.add(1) };
        value
    }
}

impl<'a, T: 'a> Iterator for SliceIterPtr<'a, T> {
    type Item = *const T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match !self.is_finished() {
            true => Some(unsafe { self.next_unchecked() }),
            false => None,
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe { self.exclusive_end.offset_from(self.data) as usize };
        (len, Some(len))
    }
}

impl<'a, T: 'a> ExactSizeIterator for SliceIterPtr<'a, T> {
    fn len(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }
}
