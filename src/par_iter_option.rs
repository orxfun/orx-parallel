use crate::{ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::sync::atomic::{AtomicBool, Ordering};

pub trait ParIterOption<T>: ParIter<Item = Option<T>> {
    fn collect_option_into<C>(self, output: C) -> Option<C>
    where
        C: ParCollectInto<T>,
    {
        let has_none = AtomicBool::new(false);

        let result = self
            .inspect(|x| {
                if x.is_none() {
                    _ = has_none.fetch_or(true, Ordering::Relaxed);
                }
            })
            .whilst(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect_into(output);

        match has_none.load(Ordering::Relaxed) {
            false => Some(result),
            true => None,
        }
    }

    fn collect_option<C>(self) -> Option<C>
    where
        C: ParCollectInto<T>,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_option_into(output)
    }
}

impl<P, T> ParIterOption<T> for P where P: ParIter<Item = Option<T>> {}
