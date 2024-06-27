use std::fmt::Debug;

pub trait Fallible<T> {
    fn unwrap(self) -> T;

    fn has_value(&self) -> bool;

    #[inline(always)]
    fn into_option(self) -> Option<T>
    where
        Self: Sized,
    {
        match self.has_value() {
            false => None,
            true => Some(self.unwrap()),
        }
    }
}

impl<T> Fallible<T> for Option<T> {
    #[inline(always)]
    fn unwrap(self) -> T {
        self.unwrap()
    }

    #[inline(always)]
    fn has_value(&self) -> bool {
        self.is_some()
    }
}

impl<T, E: Debug> Fallible<T> for Result<T, E> {
    #[inline(always)]
    fn unwrap(self) -> T {
        self.unwrap()
    }

    #[inline(always)]
    fn has_value(&self) -> bool {
        self.is_ok()
    }
}
