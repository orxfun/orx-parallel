use crate::composition::filter_filter::FilterFilter;
use core::marker::PhantomData;

// trait

pub trait Filter<T> {
    fn call(&self, input: &T) -> bool;
}

// unit

pub struct FilterUnit<T, F>
where
    F: Fn(&T) -> bool,
{
    f1: F,
    phantom: PhantomData<T>,
}

impl<T, F> FilterUnit<T, F>
where
    F: Fn(&T) -> bool,
{
    pub fn new(f1: F) -> Self {
        Self {
            f1,
            phantom: PhantomData,
        }
    }

    pub fn filter<B>(self, f2: B) -> FilterFilter<T, F, B>
    where
        B: Fn(&T) -> bool,
    {
        FilterFilter::new(self.f1, f2)
    }
}

impl<T, F> Filter<T> for FilterUnit<T, F>
where
    F: Fn(&T) -> bool,
{
    #[inline(always)]
    fn call(&self, input: &T) -> bool {
        (self.f1)(input)
    }
}
