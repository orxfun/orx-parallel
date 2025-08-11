use crate::computations::generalized_values::while_option::WhileOption;

pub struct WhileIterFilterMap<I, T, Fm, O>
where
    I: Iterator<Item = WhileOption<T>>,
    Fm: Fn(T) -> Option<O>,
{
    iter: I,
    filter_map: Fm,
}

impl<I, T, Fm, O> WhileIterFilterMap<I, T, Fm, O>
where
    I: Iterator<Item = WhileOption<T>>,
    Fm: Fn(T) -> Option<O>,
{
    pub fn new(iter: I, filter_map: Fm) -> Self {
        Self { iter, filter_map }
    }
}

impl<I, T, Fm, O> Iterator for WhileIterFilterMap<I, T, Fm, O>
where
    I: Iterator<Item = WhileOption<T>>,
    Fm: Fn(T) -> Option<O>,
{
    type Item = WhileOption<O>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(x) => match x.into_continue() {
                    Some(x) => match (self.filter_map)(x) {
                        Some(x) => return Some(WhileOption::continue_with(x)),
                        None => continue,
                    },
                    None => return Some(WhileOption::stop()),
                },
                None => return None,
            }
        }
    }
}
