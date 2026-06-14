use proptest::prelude::{Arbitrary, BoxedStrategy, Strategy};
use serde::Serialize;
use std::fmt::{self, Debug, Display};

/// An `Occupation` represents some value that may or may not be valid.
///
/// Consider reading in a user-specified json as example. For a given key, there are several
/// notions of (in-)valid data: a required key can be
/// - missing,
/// - present, with
///   - the specified value being invalid or
///   - the specified value being valid.
///
/// `Occupation` acts as helper to separate valid from invalid values.
/// Inspired by the terms in crystallography and condensed matter physics, a valid value is
/// represented by [`Occupation::Correct`] whereas an invalid value is represented using [`Occupation::Defect`].
///
/// To encode presence and absence, see `Site` instead.
#[derive(Debug, Serialize, Clone)]
pub enum Occupation<C, D> {
    /// A correct (valid) value.
    #[serde(untagged)]
    Correct(C),

    /// A defect (invalid) value.
    #[serde(untagged)]
    Defect(D),
}

impl<C, D> Occupation<C, D> {
    /// Get the contained correct value, if any.
    pub fn get_correct(&self) -> Option<&C> {
        match self {
            Self::Correct(c) => Some(c),
            Self::Defect(_) => None,
        }
    }

    /// Get the contained defect value, if any.
    pub fn get_defect(&self) -> Option<&D> {
        match self {
            Self::Correct(_) => None,
            Self::Defect(d) => Some(d),
        }
    }

    /// Apply `f` to the contained correct value.
    ///
    /// Defect values are left untouched.
    pub fn map_correct<T, F>(self, f: F) -> Occupation<T, D>
    where
        F: Fn(C) -> T,
    {
        match self {
            Self::Correct(c) => Occupation::Correct(f(c)),
            Self::Defect(d) => Occupation::Defect(d),
        }
    }

    /// Apply `f` to the contained defect value.
    ///
    /// Correct values are left untouched.
    pub fn map_defect<T, F>(self, f: F) -> Occupation<C, T>
    where
        F: Fn(D) -> T,
    {
        match self {
            Self::Correct(c) => Occupation::Correct(c),
            Self::Defect(d) => Occupation::Defect(f(d)),
        }
    }
}

impl<C, D> Display for Occupation<C, D>
where
    C: Display,
    D: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Correct(c) => c.fmt(f),
            Self::Defect(d) => d.fmt(f),
        }
    }
}

impl<C, D, S> Arbitrary for Occupation<C, D>
where
    C: Arbitrary<Strategy = S> + 'static,
    S: Strategy<Value = C> + 'static,
    D: Debug + 'static,
{
    type Parameters = C::Parameters;
    type Strategy = BoxedStrategy<Occupation<C, D>>;
    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        C::arbitrary_with(args)
            .prop_map(Occupation::Correct)
            .boxed()
    }
}
