use crate::computations::generalized_values::while_option::WhileOption;

pub struct WhileIterMap<I, T, M, O>
where
    I: Iterator<Item = WhileOption<T>>,
    M: Fn(T) -> O,
{
    iter: I,
    map: M,
}

impl<I, T, M, O> WhileIterMap<I, T, M, O>
where
    I: Iterator<Item = WhileOption<T>>,
    M: Fn(T) -> O,
{
    pub fn new(iter: I, map: M) -> Self {
        Self { iter, map }
    }
}

impl<I, T, M, O> Iterator for WhileIterMap<I, T, M, O>
where
    I: Iterator<Item = WhileOption<T>>,
    M: Fn(T) -> O,
{
    type Item = WhileOption<O>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.mapped(&self.map))
    }
}
