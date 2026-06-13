//! Conversion into `proptest` types.

use anyhow;
use proptest::prelude::TestCaseError;

mod seal {
    pub trait Seal {}
}

/// Local extension trait to turn `Result<T, E>` into `Result<T, TestCaseError>` in case
/// `E` does not implement `Into<TestCaseError>`
///
/// Implemented as local extension trait, as this allows calls as in `res.into_proptest()` which is
/// way more ergonomic compared to e.g. `into_proptest(res)`, as it allows for chaining.
///
/// This trait is sealed in the sense that it cannot be implemented by the user.
pub trait IntoProptest: seal::Seal {
    /// Happy case type
    type T;

    /// Convert `self`
    ///
    /// # Errors
    ///
    /// Returns `Err(TestCaseError)` in case `self` represents a failure.
    fn into_proptest(self) -> Result<Self::T, TestCaseError>;
}

fn format_anyhow(e: &anyhow::Error) -> String {
    e.chain()
        .rev()
        .skip(1)
        .fold(format!("{}", e.root_cause()), |msg, c| {
            msg + &format!("\n\t{c}")
        })
}

impl<T> seal::Seal for Result<T, anyhow::Error> {}

impl<T> IntoProptest for Result<T, anyhow::Error> {
    type T = T;

    fn into_proptest(self) -> Result<Self::T, TestCaseError> {
        self.map_err(|e| TestCaseError::fail(format_anyhow(&e)))
    }
}
