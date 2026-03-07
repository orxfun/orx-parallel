pub struct IterF<I: Iterator, F: Fn(&I::Item) -> bool> {
    i: I,
    f: F,
}

impl<I: Iterator, F: Fn(&I::Item) -> bool> IterF<I, F> {
    pub fn new(i: I, f: F) -> Self {
        Self { i, f }
    }
}

impl<I: Iterator, F: Fn(&I::Item) -> bool> Iterator for IterF<I, F> {
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // self.i.find(|x| (self.f)(&x))
        // self.i.find_map(|x| ((self.f)(&x)).then_some(x))
        loop {
            match self.i.next() {
                Some(x) => match (self.f)(&x) {
                    true => return Some(x),
                    false => {}
                },
                None => return None,
            }
        }
    }
}
