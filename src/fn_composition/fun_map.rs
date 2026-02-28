use core::marker::PhantomData;

pub trait FnMap<A, B> {
    fn call(&self, input: A) -> B;
}

pub struct FnMapWrapper<A, B, F1>
where
    F1: Fn(A) -> B,
{
    f1: F1,
    phantom: PhantomData<(A, B)>,
}

impl<A, B, F1> FnMap<A, B> for FnMapWrapper<A, B, F1>
where
    F1: Fn(A) -> B,
{
    #[inline(always)]
    fn call(&self, input: A) -> B {
        (self.f1)(input)
    }
}

pub trait Map<A, B>: FnMap<A, B> {
    type Map<M, X>: Map<A, X>
    where
        M: Map<B, X>;
}

// single

pub struct MapSingle<A, B, F1>
where
    F1: FnMap<A, B>,
{
    f1: F1,
    phantom: PhantomData<(A, B)>,
}

impl<A, B, F1> Map<A, B> for MapSingle<A, B, F1>
where
    F1: FnMap<A, B>,
{
    type Map<M, X>
        = MapPair<A, B, X, F1, M>
    where
        M: Map<B, X>;
}

impl<A, B, F1> FnMap<A, B> for MapSingle<A, B, F1>
where
    F1: FnMap<A, B>,
{
    #[inline(always)]
    fn call(&self, input: A) -> B {
        self.f1.call(input)
    }
}

// pair

pub struct MapPair<A, B, C, F1, F2>
where
    F1: FnMap<A, B>,
    F2: Map<B, C>,
{
    f1: F1,
    f2: F2,
    phantom: PhantomData<(A, B, C)>,
}

impl<A, B, C, F1, F2> Map<A, C> for MapPair<A, B, C, F1, F2>
where
    F1: FnMap<A, B>,
    F2: Map<B, C>,
{
    type Map<M, X>
        = MapPair<A, B, X, F1, F2::Map<M, X>>
    where
        M: Map<C, X>;
}

impl<A, B, C, F1, F2> FnMap<A, C> for MapPair<A, B, C, F1, F2>
where
    F1: FnMap<A, B>,
    F2: Map<B, C>,
{
    fn call(&self, input: A) -> C {
        self.f2.call(self.f1.call(input))
    }
}
