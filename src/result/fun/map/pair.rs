use crate::result::fun::map::{fn_trait::MapRes, queue::MapResQueue};

#[derive(Clone, Copy)]
pub struct ResMp<F: MapRes, B: MapResQueue<E = F::E, I = F::O>> {
    f: F,
    b: B,
}

impl<F: MapRes, B: MapResQueue<E = F::E, I = F::O>> ResMp<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: MapRes, B: MapResQueue<E = F::E, I = F::O>> MapRes for ResMp<F, B> {
    type I = F::I;

    type O = B::O;

    type E = F::E;

    #[inline(always)]
    fn map_res(&self, i: Self::I) -> Result<Self::O, Self::E> {
        self.f.map_res(i).and_then(|x| self.b.map_res(x))
    }
}

// use crate::infallible::fun::map::{Map, MapQueue};

// #[derive(Clone, Copy)]
// pub struct Mp<F: Map, B: MapQueue<I = F::O>> {
//     f: F,
//     b: B,
// }

// impl<F: Map, B: MapQueue<I = F::O>> Mp<F, B> {
//     pub fn new(f: F, b: B) -> Self {
//         Self { f, b }
//     }
// }

// impl<F: Map, B: MapQueue<I = F::O>> Map for Mp<F, B> {
//     type I = F::I;

//     type O = B::O;

//     #[inline(always)]
//     fn map(&self, i: Self::I) -> Self::O {
//         self.b.map(self.f.map(i))
//     }
// }

// impl<F: Map, B: MapQueue<I = F::O>> MapQueue for Mp<F, B> {
//     type Then<Q, H>
//         = Mp<F, B::Then<Q, H>>
//     where
//         H: Map<I = Self::O, O = Q>;

//     fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
//     where
//         H: Map<I = Self::O, O = Q>,
//     {
//         Mp::new(self.f, self.b.then(h))
//     }
// }
