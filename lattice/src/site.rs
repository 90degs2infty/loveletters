use super::occupation::Occupation;
use proptest::prelude::{Arbitrary, BoxedStrategy, Just, Strategy};
use serde::Serialize;
use std::fmt::Debug;

/// A `Site` represents some key that may or may not be present.
///
/// Consider reading in a user-specified json as example. For a given key, there are several
/// notions of (in-)valid data: a required key can be
/// - missing,
/// - present, with
///   - the specified value being invalid or
///   - the specified value being valid.
///
/// `Site` acts as helper to encode presence or absence.
/// Inspired by the terms in crystallography and condensed matter physics, a present value is
/// represented by [`Site::Occupied`] whereas an absent value is represented using [`Site::Vacant`].
///
/// # Important
///
/// When using `Site`, make sure you mark fields as `#[serde(skip_serializing_if = "Site::is_vacant")]`
/// to actually skip such fields on serialization. Not marking fields accordingly will trigger
/// `Site::Vacant cannot be serialized` errors. Due to how `serde` works internally, this skip-logic
/// cannot be represented at the type level (i.e. make fields of type `Site` disappear automagically
/// when of value `Site::Vacant`; to simplify usage by making the `skip_serializing_if` attribute
/// obsolete), so unfortunately this attribute is required on all fields of type `Site`.
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum Site<O> {
    Occupied(O),
    #[serde(skip_serializing)]
    Vacant,
}

impl<O> Site<O> {
    pub fn is_vacant(&self) -> bool {
        match self {
            Self::Vacant => true,
            _ => false,
        }
    }

    pub fn get(&self) -> Option<&O> {
        match self {
            Self::Occupied(o) => Some(&o),
            _ => None,
        }
    }

    pub fn into_option(self) -> Option<O> {
        match self {
            Site::Occupied(o) => Some(o),
            Site::Vacant => None,
        }
    }
}

impl<O: Clone + Debug> Site<O> {
    pub fn prop_vacant() -> impl Strategy<Value = Self> {
        Just(Self::Vacant)
    }

    pub fn prop_occupied(o: impl Strategy<Value = O>) -> impl Strategy<Value = Self> {
        o.prop_map(Self::Occupied)
    }
}

impl<O, S> Arbitrary for Site<O>
where
    O: Arbitrary<Strategy = S> + 'static,
    S: Strategy<Value = O> + 'static,
{
    type Parameters = O::Parameters;
    type Strategy = BoxedStrategy<Site<O>>;
    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        O::arbitrary_with(args).prop_map(Site::Occupied).boxed()
    }
}

impl<O> From<Site<O>> for Option<O> {
    fn from(value: Site<O>) -> Self {
        value.into_option()
    }
}

/// Extension trait to simplify creation of occupied [`Site`]s from custom `Strategy`s.
pub trait IntoOccupied<O>
where
    Self: Sized,
{
    fn into_occupied(self) -> impl Strategy<Value = Site<O>>;
}

impl<O, S> IntoOccupied<O> for S
where
    O: Debug + Clone,
    S: Sized + Strategy<Value = O>,
{
    fn into_occupied(self) -> impl Strategy<Value = Site<O>> {
        Site::prop_occupied(self)
    }
}

/// Short-hand for [`Site<Occupation<_, _>>`]
pub type So<C, D> = Site<Occupation<C, D>>;

impl<C, D> So<C, D> {
    pub fn get_correct(&self) -> Option<&C> {
        self.get().map(Occupation::get_correct).flatten()
    }
}

impl<C, D> So<C, D>
where
    C: Debug,
    D: Debug,
{
    pub fn prop_correct(correct: impl Strategy<Value = C>) -> impl Strategy<Value = Self> {
        correct.prop_map(|c| Self::Occupied(Occupation::Correct(c)))
    }

    pub fn prop_defect(defect: impl Strategy<Value = D>) -> impl Strategy<Value = Self> {
        defect.prop_map(|d| Self::Occupied(Occupation::Defect(d)))
    }
}

impl<C, D> So<C, D>
where
    C: Debug + Clone,
    D: Debug,
{
    pub fn prop_just(correct: C) -> impl Strategy<Value = Self> {
        Self::prop_correct(Just(correct))
    }
}

/// Extension trait to simplify creation of occupied and correct [`So`]s from custom `Strategy`s.
pub trait IntoCorrect<C, D>
where
    Self: Sized,
{
    fn into_correct(self) -> impl Strategy<Value = So<C, D>>;
}

impl<C, D, S> IntoCorrect<C, D> for S
where
    C: Debug,
    D: Debug,
    S: Sized + Strategy<Value = C>,
{
    fn into_correct(self) -> impl Strategy<Value = So<C, D>> {
        So::prop_correct(self)
    }
}

/// Extension trait to simplify creation of occupied and defect [`So`]s from custom `Strategy`s.
pub trait IntoDefect<C, D>
where
    Self: Sized,
{
    fn into_defect(self) -> impl Strategy<Value = So<C, D>>;
}

impl<C, D, S> IntoDefect<C, D> for S
where
    C: Debug,
    D: Debug,
    S: Sized + Strategy<Value = D>,
{
    fn into_defect(self) -> impl Strategy<Value = So<C, D>> {
        So::prop_defect(self)
    }
}
