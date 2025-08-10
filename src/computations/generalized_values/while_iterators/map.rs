use crate::computations::generalized_values::while_iterators::{
    while_iter::WhileIter, while_next::WhileNext,
};

pub struct WhileIterMap<I, T, M, O>
where
    I: Iterator<Item = Option<T>>,
    M: Fn(T) -> O,
{
    iter: WhileIter<I, T>,
    map: M,
}

impl<I, T, M, O> Iterator for WhileIterMap<I, T, M, O>
where
    I: Iterator<Item = Option<T>>,
    M: Fn(T) -> O,
{
    type Item = WhileNext<O>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| match x {
            WhileNext::Continue(x) => WhileNext::Continue((self.map)(x)),
            WhileNext::Stop => WhileNext::Stop,
        })
    }
}
