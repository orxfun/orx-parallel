use crate::computations::generalized_values::while_iterators::{
    while_iter::WhileIter, while_next::WhileNext,
};

pub struct WhileIterFilterMap<I, T, Fm, O>
where
    I: Iterator<Item = Option<T>>,
    Fm: Fn(T) -> Option<O>,
{
    iter: WhileIter<I, T>,
    filter_map: Fm,
}

impl<I, T, Fm, O> Iterator for WhileIterFilterMap<I, T, Fm, O>
where
    I: Iterator<Item = Option<T>>,
    Fm: Fn(T) -> Option<O>,
{
    type Item = WhileNext<O>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(x) => match x {
                    WhileNext::Continue(x) => match (self.filter_map)(x) {
                        Some(x) => return Some(WhileNext::Continue(x)),
                        None => continue,
                    },
                    WhileNext::Stop => return Some(WhileNext::Stop),
                },
                None => return None,
            }
        }
    }
}
