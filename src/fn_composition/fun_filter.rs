use core::marker::PhantomData;

// trait

pub trait FunFilter<T> {
    fn call(&self, input: &T) -> bool;
}

// unit

pub struct FunFilterUnit<T, F>
where
    F: Fn(&T) -> bool,
{
    f1: F,
    phantom: PhantomData<T>,
}

impl<T, F> FunFilter<T> for FunFilterUnit<T, F>
where
    F: Fn(&T) -> bool,
{
    #[inline(always)]
    fn call(&self, input: &T) -> bool {
        (self.f1)(input)
    }
}

// composition

pub trait Filter<T>: FunFilter<T> {
    type Filter<X>: Filter<T>
    where
        X: FunFilter<T>;
}

// single

pub struct FilterSingle<T, F>
where
    F: FunFilter<T>,
{
    f1: F,
    phantom: PhantomData<T>,
}

impl<T, F> Filter<T> for FilterSingle<T, F>
where
    F: FunFilter<T>,
{
    type Filter<X>
        = FilterPair<T, F, FilterSingle<T, X>>
    where
        X: FunFilter<T>;
}

impl<T, F> FunFilter<T> for FilterSingle<T, F>
where
    F: FunFilter<T>,
{
    #[inline(always)]
    fn call(&self, input: &T) -> bool {
        self.f1.call(input)
    }
}

// pair

pub struct FilterPair<T, F, B>
where
    F: FunFilter<T>,
    B: Filter<T>,
{
    f1: F,
    f2: B,
    phantom: PhantomData<T>,
}

impl<T, F, B> Filter<T> for FilterPair<T, F, B>
where
    F: FunFilter<T>,
    B: Filter<T>,
{
    type Filter<X>
        = FilterPair<T, F, B::Filter<X>>
    where
        X: FunFilter<T>;
}

impl<T, F, B> FunFilter<T> for FilterPair<T, F, B>
where
    F: FunFilter<T>,
    B: Filter<T>,
{
    fn call(&self, input: &T) -> bool {
        self.f1.call(input) && self.f2.call(input)
    }
}
