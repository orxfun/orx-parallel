use crate::infallible::fun::map::FnMap;
use crate::infallible::xap::Xap;
use crate::infallible::xap_variants::many_m::ManyM;
use crate::infallible::{fun::flat_map::FlatMap, size::Many};

pub struct ManyX<X: Xap<Size = Many>, G: FlatMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: FlatMap<I = X::O>> Clone for ManyX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Many>, G: FlatMap<I = X::O>> Copy for ManyX<X, G> {}

impl<X: Xap<Size = Many>, G: FlatMap<I = X::O>> ManyX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Many>, G: FlatMap<I = X::O>> Xap for ManyX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = IterManyX<<X::Values as IntoIterator>::IntoIter, G>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        let i = self.x.xap(i).into_iter();
        let (g, inner) = (self.g, None);
        IterManyX { i, g: g, inner }
    }

    // transformations

    type Map<Q, H>
        = ManyM<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        ManyM::new(self, FnMap::new(h))
    }
}

// iter

pub struct IterManyX<I, G>
where
    I: Iterator,
    G: FlatMap<I = I::Item>,
{
    i: I,
    g: G,
    inner: Option<<G::O as IntoIterator>::IntoIter>,
}

impl<I, G> Iterator for IterManyX<I, G>
where
    I: Iterator,
    G: FlatMap<I = I::Item>,
{
    type Item = <G::O as IntoIterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt;
            }

            match self.i.next() {
                Some(i) => self.inner = Some(self.g.flat_map(i).into_iter()),
                None => return None,
            }
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
