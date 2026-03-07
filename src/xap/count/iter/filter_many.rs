use crate::xap::fun::filter::FilterFn;

pub struct FilterIterMany<I: Iterator, G: FilterFn<I = I::Item>> {
    i: I,
    g: G,
}

impl<I: Iterator, G: FilterFn<I = I::Item>> FilterIterMany<I, G> {
    pub fn new(i: I, g: G) -> Self {
        Self { i, g }
    }
}

impl<I: Iterator, G: FilterFn<I = I::Item>> Iterator for FilterIterMany<I, G> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // self.i.find(|x| (self.f)(&x))
        loop {
            match self.i.next() {
                Some(x) => match self.g.filter(&x) {
                    true => return Some(x),
                    false => {}
                },
                None => return None,
            }
        }
    }
}
