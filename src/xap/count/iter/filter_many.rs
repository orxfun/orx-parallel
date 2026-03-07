use crate::xap::fun::filter::FilterFn;

pub struct FilterIterMany<I: Iterator, F: FilterFn<I = I::Item>> {
    i: I,
    f: F,
}

impl<I: Iterator, F: FilterFn<I = I::Item>> FilterIterMany<I, F> {
    pub fn new(i: I, f: F) -> Self {
        Self { i, f }
    }
}

impl<I: Iterator, F: FilterFn<I = I::Item>> Iterator for FilterIterMany<I, F> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // self.i.find(|x| (self.f)(&x))
        loop {
            match self.i.next() {
                Some(x) => match self.f.filter(&x) {
                    true => return Some(x),
                    false => {}
                },
                None => return None,
            }
        }
    }
}
