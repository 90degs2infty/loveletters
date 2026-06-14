//! Helper to represent pattern match failures.

/// An unexpected value not matching a certain pattern.
#[derive(Debug)]
pub struct Mismatch<T> {
    inner: T,
    pattern: String,
}

impl<T> std::fmt::Display for Mismatch<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { inner, pattern } = self;
        write!(f, "value {:?} did not match pattern {}", inner, pattern)
    }
}

impl<T> std::error::Error for Mismatch<T> where T: std::fmt::Debug {}

impl<T> Mismatch<T> {
    /// Create a new [`Mismatch`] value indicating `value` does not match `pattern`.
    pub fn new(value: T, pattern: String) -> Self {
        Self {
            inner: value,
            pattern,
        }
    }
}

/// Helper to turn a match statement into an [`Result<(), Mismatch<T>>`].
///
/// This macro matches the first argument `$e` against the second argument `$pat` (i.e. the latter
/// being the pattern).
///
/// Also see [`Mismatch`].
///
/// # Errors
///
/// In case the match succeeds, `Ok(())` is returned.
/// In case the match does not succeed, an `Err(Mismatch)` is returned with both the original
/// value `$e` as well as the stringified version of `$pat` wrapped.
#[macro_export]
macro_rules! try_match {
    ( $e:expr , $pat:pat ) => {{
        match $e {
            $pat => Ok(()),
            r => Err(Mismatch::new(r, String::from(stringify!($pat)))),
        }
    }};
}
