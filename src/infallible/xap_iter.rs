use crate::infallible::Xap;

pub struct XapIter<I, X>
where
    I: Iterator,
    X: Xap<I = I::Item>,
{
    iter: I,
    xap: X,
}

impl<I, X> Iterator for XapIter<I, X>
where
    I: Iterator,
    X: Xap<I = I::Item>,
{
    type Item = X::O;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
