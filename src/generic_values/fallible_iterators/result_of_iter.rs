pub struct ResultOfIter<I, E>
where
    I: Iterator,
{
    iter: Option<I>,
    error: Option<E>,
}

impl<I, E> ResultOfIter<I, E>
where
    I: Iterator,
{
    pub fn ok(iter: I) -> Self {
        Self {
            iter: Some(iter),
            error: None,
        }
    }

    pub fn err(error: E) -> Self {
        Self {
            iter: None,
            error: Some(error),
        }
    }
}

impl<I, E> Iterator for ResultOfIter<I, E>
where
    I: Iterator,
{
    type Item = Result<I::Item, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.as_mut() {
            Some(iter) => iter.next().map(|x| Ok(x)),
            None => self.error.take().map(|e| Err(e)),
        }
    }
}
