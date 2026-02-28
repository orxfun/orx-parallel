use crate::fun_composition::xap::fn_xap::Xap;
use core::marker::PhantomData;

pub struct FilterMap<I, O, F, M>
where
    F: Fn(&I) -> bool,
    M: Fn(I) -> O,
{
    f: F,
    m: M,
    phantom: PhantomData<I>,
}

impl<I, O, F, M> Xap<I> for FilterMap<I, O, F, M>
where
    F: Fn(&I) -> bool,
    M: Fn(I) -> O,
{
    type O = Option<O>;

    fn run(&self, i: I) -> Self::O {
        match (self.f)(&i) {
            true => Some((self.m)(i)),
            false => None,
        }
    }
}
