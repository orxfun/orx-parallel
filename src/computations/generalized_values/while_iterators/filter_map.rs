use crate::computations::generalized_values::while_iterators::while_next::WhileNext;

pub struct WhileIterFilterMap<I, T, Fm, O>
where
    I: Iterator<Item = WhileNext<T>>,
    Fm: Fn(T) -> Option<O>,
{
    iter: I,
    filter_map: Fm,
}

impl<I, T, Fm, O> WhileIterFilterMap<I, T, Fm, O>
where
    I: Iterator<Item = WhileNext<T>>,
    Fm: Fn(T) -> Option<O>,
{
    pub fn new(iter: I, filter_map: Fm) -> Self {
        Self { iter, filter_map }
    }
}

impl<I, T, Fm, O> Iterator for WhileIterFilterMap<I, T, Fm, O>
where
    I: Iterator<Item = WhileNext<T>>,
    Fm: Fn(T) -> Option<O>,
{
    type Item = WhileNext<O>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(x) => match x.into_continue() {
                    Some(x) => match (self.filter_map)(x) {
                        Some(x) => return Some(WhileNext::continue_with(x)),
                        None => continue,
                    },
                    None => return Some(WhileNext::stop()),
                },
                None => return None,
            }
        }
    }
}
