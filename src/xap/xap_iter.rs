use crate::xap::Xap;

pub struct XapIter<'i, I: Iterator, X: Xap<I = I::Item> + 'i> {
    i: I,
    x: X,
    inner: Option<<X::Values<'i> as IntoIterator>::IntoIter>,
}

impl<'i, I: Iterator, X: Xap<I = I::Item> + 'i> XapIter<'i, I, X> {
    pub fn new(i: I, x: X) -> Self {
        let inner = None;
        Self { i, x, inner }
    }
}

impl<'i, I: Iterator, X: Xap<I = I::Item> + 'i> Iterator for XapIter<'i, I, X> {
    type Item = X::O;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt;
            }

            // match self.i.next() {
            //     Some(i) => self.inner = Some(self.x.xap(i).into_iter()),
            //     None => return None,
            // }
            return todo!();
        }
    }
}

#[inline(always)]
fn and_then_or_clear<T, U>(opt: &mut Option<T>, f: impl FnOnce(&mut T) -> Option<U>) -> Option<U> {
    let x = f(opt.as_mut()?);
    if x.is_none() {
        *opt = None;
    }
    x
}
