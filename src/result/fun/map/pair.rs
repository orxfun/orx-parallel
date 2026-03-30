use crate::result::fun::map::{fn_trait::MapRes, queue::MapResQueue};

#[derive(Clone, Copy)]
pub struct Mp<F: MapRes, B: MapResQueue<I = F::O, E = F::E>> {
    f: F,
    b: B,
}

impl<F: MapRes, B: MapResQueue<I = F::O, E = F::E>> Mp<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: MapRes, B: MapResQueue<I = F::O, E = F::E>> MapRes for Mp<F, B> {
    type I = F::I;

    type O = B::O;

    type E = F::E;

    fn map(&self, i: Self::I) -> Result<Self::O, Self::E> {
        match self.f.map(i) {
            Ok(x) => self.b.map(x),
            Err(e) => Err(e),
        }
    }
}
