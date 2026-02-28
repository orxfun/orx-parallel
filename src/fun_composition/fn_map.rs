use core::marker::PhantomData;

pub trait FnMap {
    type I;

    type O;

    fn run(&self, i: Self::I) -> Self::O;
}

pub struct FnMapUnit<I, O, F>
where
    F: Fn(I) -> O,
{
    f: F,
    phantom: PhantomData<(I, O)>,
}

impl<I, O, F> FnMap for FnMapUnit<I, O, F>
where
    F: Fn(I) -> O,
{
    type I = I;

    type O = O;

    fn run(&self, i: Self::I) -> Self::O {
        (self.f)(i)
    }
}

// queue

pub trait MapQ: FnMap {
    type Front;

    type Back: MapQ;

    type PushBack<Elem>: MapQ<I = Self::I, O = Elem::O>
    where
        Elem: FnMap<I = Self::O>;
}

// queue - single

pub struct MapQSingle<F: FnMap> {
    f: F,
}

impl<F: FnMap> MapQ for MapQSingle<F> {
    type Front = F;

    type Back = Self;

    type PushBack<Elem>
        = MapQPair<F, MapQSingle<Elem>>
    where
        Elem: FnMap<I = Self::O>;
}

impl<F: FnMap> FnMap for MapQSingle<F> {
    type I = F::I;

    type O = F::O;

    fn run(&self, i: Self::I) -> Self::O {
        self.f.run(i)
    }
}

// queue - pair

pub struct MapQPair<F: FnMap, B: MapQ>
where
    B: FnMap<I = F::O>,
{
    f: F,
    b: B,
}

impl<F: FnMap, B: MapQ> MapQ for MapQPair<F, B>
where
    B: FnMap<I = F::O>,
{
    type Front = F;

    type Back = B;

    type PushBack<Elem>
        = MapQPair<F, B::PushBack<Elem>>
    where
        Elem: FnMap<I = Self::O>;
}

impl<F: FnMap, B: MapQ> FnMap for MapQPair<F, B>
where
    B: FnMap<I = F::O>,
{
    type I = F::I;

    type O = B::O;

    fn run(&self, i: Self::I) -> Self::O {
        self.b.run(self.f.run(i))
    }
}
