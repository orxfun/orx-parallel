use crate::{ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_option::{ConcurrentOption, IntoOption};
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub trait ParIterResult<T, E>: ParIter<Item = Result<T, E>> {
    fn collect_result_into<C>(self, output: C) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync,
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
            .take_while(|x| x.is_some())
            .map(|x| {
                // SAFETY: since x passed the whilst(is-some) check, unwrap_unchecked
                unsafe { x.unwrap_unchecked() }
            })
            .collect_into(output);

        match error.into_option() {
            None => Ok(result),
            Some(e) => Err(e),
        }
    }

    fn collect_result<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_result_into(output)
    }
}

impl<P, T, E> ParIterResult<T, E> for P where P: ParIter<Item = Result<T, E>> {}

pub trait ParIterResultNew<T, E>: ParIter<Item = Result<T, E>>
where
    T: Send,
    E: Send + Sync,
{
    fn collect_result_into_new<C>(self, output: C) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync;

    fn collect_result_new<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_result_into_new(output)
    }
}
