use crate::runner::{DefaultRunner, ParRunner, default_runner};
use crate::xap::{Id, Xap};
use orx_concurrent_iter::ConcurrentIter;

// TODO: this will be replaced later by IntoPar trait.
pub fn par<I: ConcurrentIter>(iter: I) -> Par<I, Id<I::Item>, DefaultRunner> {
    Par {
        iter,
        xap: Id::new(),
        exe: default_runner(),
    }
}

pub struct Par<I, X, R = DefaultRunner>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    iter: I,
    xap: X,
    exe: R,
}

impl<I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    fn new(iter: I, xap: X, exe: R) -> Self {
        Self { iter, xap, exe }
    }

    // transformations

    pub fn map<Q, H>(self, h: H) -> Par<I, X::Map<Q, H>, R>
    where
        H: Fn(X::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        Par::new(self.iter, xap, self.exe)
    }

    pub fn inspect<H>(self, h: H) -> Par<I, X::Inspect<H>, R>
    where
        H: Fn(&X::O) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        Par::new(self.iter, xap, self.exe)
    }

    pub fn filter<H>(self, h: H) -> Par<I, X::Filter<H>, R>
    where
        H: Fn(&X::O) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        Par::new(self.iter, xap, self.exe)
    }

    pub fn filter_map<Q, H>(self, h: H) -> Par<I, X::FilterMap<Q, H>, R>
    where
        H: Fn(X::O) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        Par::new(self.iter, xap, self.exe)
    }

    pub fn flat_map<V, H>(self, h: H) -> Par<I, X::FlatMap<V, H>, R>
    where
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        Par::new(self.iter, xap, self.exe)
    }
}
