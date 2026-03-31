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

    /// Returns either of the following:
    ///
    /// * Some(Some(found)): no failure, found an element
    /// * Some(None): no failure but no element to find
    /// * None: a failure (None) is observed
    pub fn first_opt<T>(results: Vec<Option<Option<T>>>) -> Option<Option<T>> {
        for x in results {
            match x {
                Some(Some(x)) => return Some(Some(x)),
                Some(None) => {}
                None => return None,
            }
        }
        Some(None)
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

    /// Returns either of the following:
    ///
    /// * Some(Some(aggregate)): no failure, returns the aggregate
    /// * Some(None): no failure but also no element to reduce
    /// * None: a failure (None) is observed
    pub fn reduce_opt<T, F>(results: Vec<Option<Option<T>>>, f: F) -> Option<Option<T>>
    where
        F: Fn(T, T) -> T,
    {
        let mut acc = None;

        for x in results {
            match (acc.is_some(), x) {
                (true, Some(Some(x))) => acc = acc.map(|y| f(y, x)),
                (false, Some(Some(x))) => acc = Some(x),
                (_, Some(None)) => {}
                (_, None) => return None,
            }
        }

        Some(acc)
    }
}
