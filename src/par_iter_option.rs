use crate::{ParCollectInto, ParIter};
use std::sync::atomic::{AtomicBool, Ordering};

pub trait ParIterOption<T>: ParIter<Item = Option<T>> {
    fn collect_option<C>(self) -> Option<C>
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
            .collect::<C>();

        match has_none.load(Ordering::Relaxed) {
            false => Some(result),
            true => None,
        }
    }
}

impl<P, T> ParIterOption<T> for P where P: ParIter<Item = Option<T>> {}
