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

    pub fn first_res<T, E>(results: Vec<Result<Option<T>, E>>) -> Result<Option<T>, E> {
        for x in results {
            match x {
                Ok(Some(x)) => return Ok(Some(x)),
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(None)
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

    pub fn reduce_res<T, E, F>(results: Vec<Result<Option<T>, E>>, f: F) -> Result<Option<T>, E>
    where
        F: Fn(T, T) -> T,
    {
        let mut acc = None;

        for x in results {
            match (acc.is_some(), x) {
                (true, Ok(Some(x))) => acc = acc.map(|y| f(y, x)),
                (false, Ok(Some(x))) => acc = Some(x),
                (_, Ok(None)) => {}
                (_, Err(e)) => return Err(e),
            }
        }

        Ok(acc)
    }
}
