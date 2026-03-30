use alloc::vec::Vec;

pub struct Val;

impl Val {
    pub fn first<T>(results: Vec<Option<T>>) -> Option<T> {
        for x in results {
            if x.is_some() {
                return x;
            }
        }
        None
    }

    pub fn reduce<T, F>(results: Vec<Option<T>>, f: F) -> Option<T>
    where
        F: Fn(T, T) -> T,
    {
        let mut acc = None;

        for x in results {
            match (acc.is_some(), x) {
                (true, Some(x)) => acc = acc.map(|y| f(y, x)),
                (false, Some(x)) => acc = Some(x),
                (_, None) => {}
            }
        }

        acc
    }
}
