use core::marker::PhantomData;

// trait

pub trait FunMap<T1, T2> {
    fn call(&self, input: T1) -> T2;
}

// unit

pub struct FunMapUnit<T1, T2, F>
where
    F: Fn(T1) -> T2,
{
    f1: F,
    phantom: PhantomData<(T1, T2)>,
}

impl<T1, T2, F> FunMap<T1, T2> for FunMapUnit<T1, T2, F>
where
    F: Fn(T1) -> T2,
{
    #[inline(always)]
    fn call(&self, input: T1) -> T2 {
        (self.f1)(input)
    }
}

// composition

pub trait Map<T1, T2>: FunMap<T1, T2> {
    type Map<T, X>: Map<T1, T>
    where
        X: FunMap<T2, T>;
    fn map<T, X>(self, x: X) -> Self::Map<T, X>
    where
        X: FunMap<T2, T>;
}

// single

pub struct MapSingle<T1, T2, F>
where
    F: FunMap<T1, T2>,
{
    f1: F,
    phantom: PhantomData<(T1, T2)>,
}

impl<T1, T2, F> Map<T1, T2> for MapSingle<T1, T2, F>
where
    F: FunMap<T1, T2>,
{
    type Map<T, X>
        = MapPair<T1, T2, T, F, MapSingle<T2, T, X>>
    where
        X: FunMap<T2, T>;
    fn map<T, X>(self, x: X) -> Self::Map<T, X>
    where
        X: FunMap<T2, T>,
    {
        MapPair {
            f1: self.f1,
            f2: MapSingle {
                f1: x,
                phantom: PhantomData,
            },
            phantom: PhantomData,
        }
    }
}

impl<T1, T2, F> FunMap<T1, T2> for MapSingle<T1, T2, F>
where
    F: FunMap<T1, T2>,
{
    #[inline(always)]
    fn call(&self, input: T1) -> T2 {
        self.f1.call(input)
    }
}

// pair

pub struct MapPair<T1, T2, T3, F, B>
where
    F: FunMap<T1, T2>,
    B: Map<T2, T3>,
{
    f1: F,
    f2: B,
    phantom: PhantomData<(T1, T2, T3)>,
}

impl<T1, T2, T3, F, B> Map<T1, T3> for MapPair<T1, T2, T3, F, B>
where
    F: FunMap<T1, T2>,
    B: Map<T2, T3>,
{
    type Map<T, X>
        = MapPair<T1, T2, T, F, B::Map<T, X>>
    where
        X: FunMap<T3, T>;
    fn map<T, X>(self, x: X) -> Self::Map<T, X>
    where
        X: FunMap<T3, T>,
    {
        MapPair {
            f1: self.f1,
            f2: self.f2.map(x),
            phantom: PhantomData,
        }
    }
}

impl<T1, T2, T3, F, B> FunMap<T1, T3> for MapPair<T1, T2, T3, F, B>
where
    F: FunMap<T1, T2>,
    B: Map<T2, T3>,
{
    fn call(&self, input: T1) -> T3 {
        self.f2.call(self.f1.call(input))
    }
}
