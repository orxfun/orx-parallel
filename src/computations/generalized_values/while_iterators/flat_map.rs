use crate::computations::generalized_values::while_iterators::{
    while_iter::WhileIter, while_next::WhileNext,
};

pub struct WhileIterFlatMap<I, T, Fm, Io>
where
    I: Iterator<Item = Option<T>>,
    Fm: Fn(T) -> Io,
    Io: IntoIterator,
{
    iter: WhileIter<I, T>,
    filter_map: Fm,
    inner_iter: Option<Io::IntoIter>,
}

impl<I, T, Fm, Io> WhileIterFlatMap<I, T, Fm, Io>
where
    I: Iterator<Item = Option<T>>,
    Fm: Fn(T) -> Io,
    Io: IntoIterator,
{
    fn next_inner(&mut self) -> Option<WhileNext<Io::Item>> {
        match self.iter.next() {
            Some(x) => match x {
                WhileNext::Continue(x) => {
                    let inner_iter = (self.filter_map)(x);
                    self.inner_iter = Some(inner_iter.into_iter());
                    self.next()
                }
                WhileNext::Stop => Some(WhileNext::Stop),
            },
            None => None,
        }
    }
}

impl<I, T, Fm, Io> Iterator for WhileIterFlatMap<I, T, Fm, Io>
where
    I: Iterator<Item = Option<T>>,
    Fm: Fn(T) -> Io,
    Io: IntoIterator,
{
    type Item = WhileNext<Io::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner_iter {
            Some(x) => match x.next() {
                Some(x) => Some(WhileNext::Continue(x)),
                None => self.next_inner(),
            },
            None => self.next_inner(),
        }
    }
}
