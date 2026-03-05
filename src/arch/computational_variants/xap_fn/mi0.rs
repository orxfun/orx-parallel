use core::marker::PhantomData;

use crate::computational_variants::xap_fn::map::Map;

pub struct Mi0<I, F>
where
    F: Fn(&I),
{
    f: F,
    p: PhantomData<I>,
}

impl<I, F> Mi0<I, F>
where
    F: Fn(&I),
{
    pub fn new(f: F) -> Self {
        let p = PhantomData;
        Self { f, p }
    }
}

// impl<I, F> Map<I, I> for Mi0<I, F>
// where
//     F: Fn(&I),
// {
//     #[inline(always)]
//     fn map(&self, i: I) -> I {
//         (self.f)(&i);
//         i
//     }

//     type Compose<Y, Q>
//     where
//         Y: Fn(I) -> Q;

//     fn compose<Y, Q>(self, y: Y) -> Self::Compose<Y, Q>
//     where
//         Y: Fn(I) -> Q,
//     {
//         todo!()
//     }
// }
