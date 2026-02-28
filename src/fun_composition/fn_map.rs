use core::marker::PhantomData;

pub trait FnMap {
    type I;

    type O;

    fn run(&self, i: Self::I) -> Self::O;
}

pub struct WrMap<I, O, F>
where
    F: Fn(I) -> O,
{
    f: F,
    phantom: PhantomData<(I, O)>,
}

impl<I, O, F> WrMap<I, O, F>
where
    F: Fn(I) -> O,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

impl<I, O, F> FnMap for WrMap<I, O, F>
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

    type Pb<Elem>: MapQ<I = Self::I, O = Elem::O>
    where
        Elem: FnMap<I = Self::O>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
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

    type Pb<Elem>
        = MapQPair<F, MapQSingle<Elem>>
    where
        Elem: FnMap<I = Self::O>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
    where
        Elem: FnMap<I = Self::O>,
    {
        MapQPair {
            f: self.f,
            b: MapQSingle { f: elem },
        }
    }
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

    type Pb<Elem>
        = MapQPair<F, B::Pb<Elem>>
    where
        Elem: FnMap<I = Self::O>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
    where
        Elem: FnMap<I = Self::O>,
    {
        MapQPair {
            f: self.f,
            b: self.b.push_back(elem),
        }
    }
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
