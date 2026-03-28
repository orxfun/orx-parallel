use crate::infallible::Xap;

pub struct IdRes<I, E, X: Xap<O = Result<I, E>>>(X);

impl<I, E, X: Xap<O = Result<I, E>>> Clone for IdRes<I, E, X> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<I, E, X: Xap<O = Result<I, E>>> Copy for IdRes<I, E, X> {}

unsafe impl<I, E, X: Xap<O = Result<I, E>>> Send for IdRes<I, E, X> {}
