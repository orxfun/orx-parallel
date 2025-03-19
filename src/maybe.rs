use std::fmt::Debug;

/// Trait representing results which might be missing.
/// This trait is useful in defining a common interface for [`crate::Par::filter_map`] inputs.
///
/// Two common examples of types which might not have a success value are `Option` and `Result` which might or not have a successful value.
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
/// let absent: Option<char> = None;
/// assert!(!absent.has_value());
///
/// let absent: Result<char, String> = Err("failed".to_string());
/// assert!(!fail.has_value());
/// ```
pub trait Maybe<T> {
    /// Returns the successful value of the maybe type; panics if there is no successful value.
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
    /// let absent: Option<char> = None;
    /// // let _ = absent.value(); // panics!
    ///
    /// let absent: Result<char, String> = Err("failed".to_string());
    /// // let _ = absent.value(); // panics!
    /// ```
    fn unwrap(self) -> T;

    /// Returns whether or not the maybe has a successful value.
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
    /// let absent: Option<char> = None;
    /// assert!(!absent.has_value());
    ///
    /// let absent: Result<char, String> = Err("failed".to_string());
    /// assert!(!absent.has_value());
    /// ```
    fn has_value(&self) -> bool;

    fn has_value_and(&self, filter: impl Fn(&T) -> bool) -> bool;

    /// Converts the maybe into an option.
    ///
    /// Returns
    ///
    /// * `Some(self.value())` if `self.has_value()` is true,
    /// * `None` otherwise.
    fn into_option(self) -> Option<T>;
}

impl<T> Maybe<T> for Option<T> {
    #[inline(always)]
    fn unwrap(self) -> T {
        self.expect("`value` called on the variant where the success value is absent (None).")
    }

    #[inline(always)]
    fn has_value(&self) -> bool {
        self.is_some()
    }

    #[inline(always)]
    fn has_value_and(&self, filter: impl Fn(&T) -> bool) -> bool {
        self.as_ref().map(filter).unwrap_or(false)
    }

    #[inline(always)]
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<T, E: Debug> Maybe<T> for Result<T, E> {
    #[inline(always)]
    fn unwrap(self) -> T {
        self.expect("`value` called on the variant where the success value is absent (Err).")
    }

    #[inline(always)]
    fn has_value(&self) -> bool {
        self.is_ok()
    }

    #[inline(always)]
    fn has_value_and(&self, filter: impl Fn(&T) -> bool) -> bool {
        self.as_ref().map(filter).unwrap_or(false)
    }

    #[inline(always)]
    fn into_option(self) -> Option<T> {
        self.ok()
    }
}
