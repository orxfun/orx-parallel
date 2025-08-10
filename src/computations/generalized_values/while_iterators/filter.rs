use crate::computations::generalized_values::while_iterators::{
    while_iter::WhileIter, while_next::WhileNext,
};

pub struct WhileIterFilter<I, T, F>
where
    I: Iterator<Item = Option<T>>,
    F: Fn(&T) -> bool,
{
    iter: WhileIter<I, T>,
    filter: F,
}

impl<I, T, F> Iterator for WhileIterFilter<I, T, F>
where
    I: Iterator<Item = Option<T>>,
    F: Fn(&T) -> bool,
{
    type Item = WhileNext<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(x) => match x {
                    WhileNext::Continue(x) => match (self.filter)(&x) {
                        true => return Some(WhileNext::Continue(x)),
                        false => continue,
                    },
                    WhileNext::Stop => return Some(WhileNext::Stop),
                },
                None => return None,
            }
        }
    }
}
