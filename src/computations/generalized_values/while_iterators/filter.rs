use crate::computations::generalized_values::while_iterators::while_next::WhileNext;

pub struct WhileIterFilter<I, T, F>
where
    I: Iterator<Item = WhileNext<T>>,
    F: Fn(&T) -> bool,
{
    iter: I,
    filter: F,
}

impl<I, T, F> WhileIterFilter<I, T, F>
where
    I: Iterator<Item = WhileNext<T>>,
    F: Fn(&T) -> bool,
{
    pub fn new(iter: I, filter: F) -> Self {
        Self { iter, filter }
    }
}

impl<I, T, F> Iterator for WhileIterFilter<I, T, F>
where
    I: Iterator<Item = WhileNext<T>>,
    F: Fn(&T) -> bool,
{
    type Item = WhileNext<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(x) => {
                    let filtered = x.filter(&self.filter);
                    match filtered.is_some() {
                        true => return filtered,
                        false => continue,
                    }
                }
                None => return None,
            }
        }
    }
}
