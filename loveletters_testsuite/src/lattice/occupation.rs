use super::site::Site;
use proptest::prelude::{Arbitrary, BoxedStrategy, Strategy};
use serde::Serialize;
use std::fmt::{Debug, Display};

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
/// To encode presence and absence, see [`Site`] instead.
#[derive(Debug, Serialize, Clone)]
pub enum Occupation<C, D> {
    #[serde(untagged)]
    Correct(C),
    #[serde(untagged)]
    Defect(D),
}

impl<C, D> Occupation<C, D> {
    pub fn get_correct(&self) -> Option<&C> {
        match self {
            Self::Correct(c) => Some(&c),
            _ => None,
        }
    }

    pub fn get_defect(&self) -> Option<&D> {
        match self {
            Self::Defect(d) => Some(&d),
            _ => None,
        }
    }
}

impl<C, D> Display for Occupation<C, D>
where
    C: Display,
    D: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
