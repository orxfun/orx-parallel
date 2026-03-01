use crate::fun_composition::{
    fn_filter::{FilterQ, WrFilter},
    fn_map::MapQ,
    xap::fn_xap::Xap,
};
use core::marker::PhantomData;

pub struct Mf<I, O, M, F>
where
    M: MapQ<I = I, O = O>,
    F: FilterQ<I = O>,
{
    m: M,
    f: F,
    phantom: PhantomData<I>,
}

impl<I, O, M, F> Xap<I> for Mf<I, O, M, F>
where
    M: MapQ<I = I, O = O>,
    F: FilterQ<I = O>,
{
    type O = Option<O>;

    fn run(&self, i: I) -> Self::O {
        let val = self.m.run(i);
        match self.f.run(&val) {
            true => Some(val),
            false => None,
        }
    }
}

impl<I, O, M, F> Mf<I, O, M, F>
where
    M: MapQ<I = I, O = O>,
    F: FilterQ<I = O>,
{
    pub fn new(m: M, f: F) -> Self {
        Self {
            m,
            f,
            phantom: PhantomData,
        }
    }

    pub fn filter<F2>(self, f2: F2) -> Mf<I, O, M, F::Pb<WrFilter<O, F2>>>
    where
        F2: Fn(&O) -> bool,
    {
        let f = self.f.push_back(WrFilter::new(f2));
        Mf::new(self.m, f)
    }
}

// trait

pub trait FnMapFilter {
    type I;

    type O;

    fn run(&self, i: Self::I) -> Self::O;
}

impl<I, O, M, F> FnMapFilter for Mf<I, O, M, F>
where
    M: MapQ<I = I, O = O>,
    F: FilterQ<I = O>,
{
    type I = I;

    type O = Option<O>;

    fn run(&self, i: I) -> Self::O {
        let val = self.m.run(i);
        match self.f.run(&val) {
            true => Some(val),
            false => None,
        }
    }
}

// queue

pub trait MapFilterQ: FnMapFilter {
    type Front;

    type Back: MapFilterQ;

    type Pb<Elem>: MapFilterQ<I = Self::I, O = Elem::O>
    where
        Elem: FnMapFilter<I = Self::O>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
    where
        Elem: FnMapFilter<I = Self::O>;
}

// queue - single

pub struct MapFilterQSingle<F: FnMapFilter> {
    f: F,
}

impl<F: FnMapFilter> From<F> for MapFilterQSingle<F> {
    fn from(f: F) -> Self {
        Self { f }
    }
}

impl<F: FnMapFilter> MapFilterQ for MapFilterQSingle<F> {
    type Front = F;

    type Back = Self;

    type Pb<Elem>
        = MapQPair<F, MapFilterQSingle<Elem>>
    where
        Elem: FnMapFilter<I = Self::O>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
    where
        Elem: FnMapFilter<I = Self::O>,
    {
        MapQPair {
            f: self.f,
            b: MapFilterQSingle { f: elem },
        }
    }
}

impl<F: FnMapFilter> FnMapFilter for MapFilterQSingle<F> {
    type I = F::I;

    type O = F::O;

    fn run(&self, i: Self::I) -> Self::O {
        self.f.run(i)
    }
}

// queue - pair

pub struct MapQPair<F: FnMapFilter, B: MapFilterQ>
where
    B: FnMapFilter<I = F::O>,
{
    f: F,
    b: B,
}

impl<F: FnMapFilter, B: MapFilterQ> MapFilterQ for MapQPair<F, B>
where
    B: FnMapFilter<I = F::O>,
{
    type Front = F;

    type Back = B;

    type Pb<Elem>
        = MapQPair<F, B::Pb<Elem>>
    where
        Elem: FnMapFilter<I = Self::O>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
    where
        Elem: FnMapFilter<I = Self::O>,
    {
        MapQPair {
            f: self.f,
            b: self.b.push_back(elem),
        }
    }
}

impl<F: FnMapFilter, B: MapFilterQ> FnMapFilter for MapQPair<F, B>
where
    B: FnMapFilter<I = F::O>,
{
    type I = F::I;

    type O = B::O;

    fn run(&self, i: Self::I) -> Self::O {
        self.b.run(self.f.run(i))
    }
}
