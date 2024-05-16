use std::{cmp::Ordering, ops::Add};

pub trait Reduce<T>
where
    Self: Sized,
    T: Send + Sync,
{
    fn reduce<R>(self, reduce: R) -> Option<T>
    where
        R: Fn(T, T) -> T + Send + Sync;

    fn fold<Id, F>(self, identity: Id, fold: F) -> T
    where
        Id: Fn() -> T,
        F: Fn(T, T) -> T + Send + Sync,
    {
        self.reduce(fold).unwrap_or_else(identity)
    }

    fn sum(self) -> T
    where
        T: Default + Add<Output = T>,
    {
        self.reduce(|x, y| x + y).unwrap_or(T::default())
    }

    fn min(self) -> Option<T>
    where
        T: Ord,
    {
        self.reduce(|x, y| if x <= y { x } else { y })
    }

    fn max(self) -> Option<T>
    where
        T: Ord,
    {
        self.reduce(|x, y| if x >= y { x } else { y })
    }

    fn min_by<F>(self, compare: F) -> Option<T>
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.reduce(|x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        })
    }

    fn max_by<F>(self, compare: F) -> Option<T>
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.reduce(|x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        })
    }

    fn min_by_key<B, F>(self, get_key: F) -> Option<T>
    where
        B: Ord,
        F: Fn(&T) -> B + Sync,
    {
        self.reduce(|x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        })
    }

    fn max_by_key<B, F>(self, get_key: F) -> Option<T>
    where
        B: Ord,
        F: Fn(&T) -> B + Sync,
    {
        self.reduce(|x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        })
    }
}
