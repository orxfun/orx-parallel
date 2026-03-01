use core::marker::PhantomData;

pub trait FnFilter {
    type I;

    fn run(&self, i: &Self::I) -> bool;
}

pub struct WrFilter<I, F>
where
    F: Fn(&I) -> bool,
{
    f: F,
    phantom: PhantomData<I>,
}

impl<I, F> WrFilter<I, F>
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

impl<I, F> FnFilter for WrFilter<I, F>
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

    type Pb<Elem>: FilterQ<I = Self::I>
    where
        Elem: FnFilter<I = Self::I>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
    where
        Elem: FnFilter<I = Self::I>;
}

// queue - single

pub struct FilterQSingle<F: FnFilter> {
    f: F,
}

impl<F: FnFilter> From<F> for FilterQSingle<F> {
    fn from(f: F) -> Self {
        Self { f }
    }
}

impl<F: FnFilter> FilterQ for FilterQSingle<F> {
    type Front = F;

    type Back = Self;

    type Pb<Elem>
        = FilterQPair<F, FilterQSingle<Elem>>
    where
        Elem: FnFilter<I = Self::I>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
    where
        Elem: FnFilter<I = Self::I>,
    {
        FilterQPair {
            f: self.f,
            b: FilterQSingle { f: elem },
        }
    }
}

impl<F: FnFilter> FnFilter for FilterQSingle<F> {
    type I = F::I;

    fn run(&self, i: &Self::I) -> bool {
        self.f.run(i)
    }
}

// queue - pair

pub struct FilterQPair<F: FnFilter, B: FilterQ>
where
    B: FnFilter<I = F::I>,
{
    f: F,
    b: B,
}

impl<F: FnFilter, B: FilterQ> FilterQ for FilterQPair<F, B>
where
    B: FnFilter<I = F::I>,
{
    type Front = F;

    type Back = B;

    type Pb<Elem>
        = FilterQPair<F, B::Pb<Elem>>
    where
        Elem: FnFilter<I = Self::I>;
    fn push_back<Elem>(self, elem: Elem) -> Self::Pb<Elem>
    where
        Elem: FnFilter<I = Self::I>,
    {
        FilterQPair {
            f: self.f,
            b: self.b.push_back(elem),
        }
    }
}

impl<F: FnFilter, B: FilterQ> FnFilter for FilterQPair<F, B>
where
    B: FnFilter<I = F::I>,
{
    type I = F::I;

    fn run(&self, i: &Self::I) -> bool {
        self.f.run(i) && self.b.run(i)
    }
}
