use crate::computations::generalized_values::while_iterators::while_next::WhileNext;

pub struct WhileIter<I, T>
where
    I: Iterator<Item = Option<T>>,
{
    iter: I,
}

impl<I, T> Iterator for WhileIter<I, T>
where
    I: Iterator<Item = Option<T>>,
{
    type Item = WhileNext<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| match x {
            Some(x) => WhileNext::Continue(x),
            None => WhileNext::Stop,
        })
    }
}
