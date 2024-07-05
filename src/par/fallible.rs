use std::fmt::Debug;

/// Trait representing results which might be missing.
/// This trait is useful in defining a common interface for [`crate::ParIter::filter_map`] inputs.
///
/// Two common examples of fallible types are `Option` and `Result` which might or not have a successful value.
///
/// # Example
///
/// ```rust
/// use orx_parallel::*;
///
/// let success = Some(42);
/// assert!(success.has_value());
/// assert_eq!(success.value(), 42);
///
/// let success: Result<_, String> = Ok(42);
/// assert!(success.has_value());
/// assert_eq!(success.value(), 42);
///
/// let fail: Option<char> = None;
/// assert!(!fail.has_value());
///
/// let fail: Result<char, String> = Err("failed".to_string());
/// assert!(!fail.has_value());
/// ```
pub trait Fallible<T> {
    /// Returns the successful value of the fallible type; panics if there is no successful value.
    ///
    /// # Panics
    ///
    /// Panics when `value` is called when `has_value` is false.
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let success = Some(42);
    /// assert_eq!(success.value(), 42);
    ///
    /// let success: Result<_, String> = Ok(42);
    /// assert_eq!(success.value(), 42);
    ///
    /// let fail: Option<char> = None;
    /// // let _ = fail.value(); // panics!
    ///
    /// let fail: Result<char, String> = Err("failed".to_string());
    /// // let _ = fail.value(); // panics!
    /// ```
    fn value(self) -> T;

    /// Returns whether or not the fallible has a successful value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let success = Some(42);
    /// assert!(success.has_value());
    ///
    /// let success: Result<_, String> = Ok(42);
    /// assert!(success.has_value());
    ///
    /// let fail: Option<char> = None;
    /// assert!(!fail.has_value());
    ///
    /// let fail: Result<char, String> = Err("failed".to_string());
    /// assert!(!fail.has_value());
    /// ```
    fn has_value(&self) -> bool;

    /// Converts the fallible into an option.
    ///
    /// Returns
    /// * `Some(self.value())` if `self.has_value()` is true,
    /// * `None` otherwise.
    #[inline(always)]
    fn into_option(self) -> Option<T>
    where
        Self: Sized,
    {
        match self.has_value() {
            false => None,
            true => Some(self.value()),
        }
    }
}

impl<T> Fallible<T> for Option<T> {
    #[inline(always)]
    fn value(self) -> T {
        self.expect("`value` called on failure (None).")
    }

    #[inline(always)]
    fn has_value(&self) -> bool {
        self.is_some()
    }
}

impl<T, E: Debug> Fallible<T> for Result<T, E> {
    #[inline(always)]
    fn value(self) -> T {
        self.expect("`value` called on failure (Err).")
    }

    #[inline(always)]
    fn has_value(&self) -> bool {
        self.is_ok()
    }
}
