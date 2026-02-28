use core::marker::PhantomData;

pub trait FnFilter {
    type I;

    fn run(&self, i: &Self::I) -> bool;
}

pub struct FnFilterUnit<I, F>
where
    F: Fn(&I) -> bool,
{
    f: F,
    phantom: PhantomData<I>,
}

impl<I, F> FnFilterUnit<I, F>
where
    F: Fn(&I) -> bool,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

impl<I, F> FnFilter for FnFilterUnit<I, F>
where
    F: Fn(&I) -> bool,
{
    type I = I;

    fn run(&self, i: &Self::I) -> bool {
        (self.f)(i)
    }
}

// queue

pub trait FilterQ: FnFilter {
    type Front;

    type Back: FilterQ;

    type PushBack<Elem>: FilterQ<I = Self::I>
    where
        Elem: FnFilter<I = Self::I>;
    fn push_back<Elem>(self, elem: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnFilter<I = Self::I>;
}

// queue - single

pub struct MapQSingle<F: FnFilter> {
    f: F,
}

impl<F: FnFilter> FilterQ for MapQSingle<F> {
    type Front = F;

    type Back = Self;

    type PushBack<Elem>
        = MapQPair<F, MapQSingle<Elem>>
    where
        Elem: FnFilter<I = Self::I>;
    fn push_back<Elem>(self, elem: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnFilter<I = Self::I>,
    {
        MapQPair {
            f: self.f,
            b: MapQSingle { f: elem },
        }
    }
}

impl<F: FnFilter> FnFilter for MapQSingle<F> {
    type I = F::I;

    fn run(&self, i: &Self::I) -> bool {
        self.f.run(i)
    }
}

// queue - pair

pub struct MapQPair<F: FnFilter, B: FilterQ>
where
    B: FnFilter<I = F::I>,
{
    f: F,
    b: B,
}

impl<F: FnFilter, B: FilterQ> FilterQ for MapQPair<F, B>
where
    B: FnFilter<I = F::I>,
{
    type Front = F;

    type Back = B;

    type PushBack<Elem>
        = MapQPair<F, B::PushBack<Elem>>
    where
        Elem: FnFilter<I = Self::I>;
    fn push_back<Elem>(self, elem: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnFilter<I = Self::I>,
    {
        MapQPair {
            f: self.f,
            b: self.b.push_back(elem),
        }
    }
}

impl<F: FnFilter, B: FilterQ> FnFilter for MapQPair<F, B>
where
    B: FnFilter<I = F::I>,
{
    type I = F::I;

    fn run(&self, i: &Self::I) -> bool {
        self.f.run(i) && self.b.run(i)
    }
}
