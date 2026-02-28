use crate::fun_composition::xap::fn_xap::Xap;
use core::marker::PhantomData;

pub struct MapFilter<I, O, M, F>
where
    M: Fn(I) -> O,
    F: Fn(&O) -> bool,
{
    m: M,
    f: F,
    phantom: PhantomData<I>,
}

impl<I, O, M, F> Xap<I> for MapFilter<I, O, M, F>
where
    M: Fn(I) -> O,
    F: Fn(&O) -> bool,
{
    type O = Option<O>;

    fn run(&self, i: I) -> Self::O {
        let val = (self.m)(i);
        match (self.f)(&val) {
            true => Some(val),
            false => None,
        }
    }
}
