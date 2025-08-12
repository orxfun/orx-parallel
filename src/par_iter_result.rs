use crate::{ParCollectInto, ParIter};
use orx_concurrent_option::{ConcurrentOption, IntoOption};

pub trait ParIterResult<T, E>: ParIter<Item = Result<T, E>> {
    fn collect_result<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Sync,
    {
        let error = ConcurrentOption::<E>::none();
        let result = self
            .map(|x| match x {
                Ok(x) => Some(x),
                Err(e) => {
                    _ = error.set_some(e);
                    None
                }
            })
            .whilst(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<C>();

        match error.into_option() {
            None => Ok(result),
            Some(e) => Err(e),
        }
    }
}

impl<P, T, E> ParIterResult<T, E> for P where P: ParIter<Item = Result<T, E>> {}
