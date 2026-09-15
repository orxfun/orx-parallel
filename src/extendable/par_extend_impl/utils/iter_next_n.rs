pub struct NextN<'a, I: Iterator> {
    iter: &'a mut I,
    n: usize,
    taken: usize,
}

impl<'a, I: Iterator> NextN<'a, I> {
    pub fn new(iter: &'a mut I, n: usize) -> Self {
        Self { iter, n, taken: 0 }
    }
}

impl<'a, I: Iterator> Iterator for NextN<'a, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.taken < self.n {
            true => {
                self.taken += 1;
                self.iter.next()
            }
            false => None,
        }
    }
}
