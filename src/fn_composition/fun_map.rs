use core::marker::PhantomData;

// trait

pub trait FunMap<A, B> {
    fn call(&self, input: A) -> B;
}

// unit

pub struct FunMapUnit<A, B, F1>
where
    F1: Fn(A) -> B,
{
    f1: F1,
    phantom: PhantomData<(A, B)>,
}

impl<A, B, F1> FunMap<A, B> for FunMapUnit<A, B, F1>
where
    F1: Fn(A) -> B,
{
    #[inline(always)]
    fn call(&self, input: A) -> B {
        (self.f1)(input)
    }
}

// composition

pub trait Map<A, B>: FunMap<A, B> {
    type Map<M, X>: Map<A, X>
    where
        M: Map<B, X>;
}

// single

pub struct MapSingle<A, B, F1>
where
    F1: FunMap<A, B>,
{
    f1: F1,
    phantom: PhantomData<(A, B)>,
}

impl<A, B, F1> Map<A, B> for MapSingle<A, B, F1>
where
    F1: FunMap<A, B>,
{
    type Map<M, X>
        = MapPair<A, B, X, F1, M>
    where
        M: Map<B, X>;
}

impl<A, B, F1> FunMap<A, B> for MapSingle<A, B, F1>
where
    F1: FunMap<A, B>,
{
    #[inline(always)]
    fn call(&self, input: A) -> B {
        self.f1.call(input)
    }
}

// pair

pub struct MapPair<A, B, C, F1, F2>
where
    F1: FunMap<A, B>,
    F2: Map<B, C>,
{
    f1: F1,
    f2: F2,
    phantom: PhantomData<(A, B, C)>,
}

impl<A, B, C, F1, F2> Map<A, C> for MapPair<A, B, C, F1, F2>
where
    F1: FunMap<A, B>,
    F2: Map<B, C>,
{
    type Map<M, X>
        = MapPair<A, B, X, F1, F2::Map<M, X>>
    where
        M: Map<C, X>;
}

impl<A, B, C, F1, F2> FunMap<A, C> for MapPair<A, B, C, F1, F2>
where
    F1: FunMap<A, B>,
    F2: Map<B, C>,
{
    fn call(&self, input: A) -> C {
        self.f2.call(self.f1.call(input))
    }
}
