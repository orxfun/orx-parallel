use core::marker::PhantomData;

use crate::composition::filter::Filter;

pub struct FilterFilter<T, F, B>
where
    F: Fn(&T) -> bool,
    B: Fn(&T) -> bool,
{
    f1: F,
    f2: B,
    phantom: PhantomData<T>,
}

impl<T, F, B> FilterFilter<T, F, B>
where
    F: Fn(&T) -> bool,
    B: Fn(&T) -> bool,
{
    pub fn new(f1: F, f2: B) -> Self {
        Self {
            f1,
            f2,
            phantom: PhantomData,
        }
    }
}

impl<T, F, B> Filter<T> for FilterFilter<T, F, B>
where
    F: Fn(&T) -> bool,
    B: Fn(&T) -> bool,
{
    #[inline(always)]
    fn call(&self, input: &T) -> bool {
        (self.f1)(input) && (self.f2)(input)
    }
}
