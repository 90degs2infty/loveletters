use proptest::prelude::*;
use std::fmt;

mod seal {
    pub trait Seal {}
}

pub trait Transpose: seal::Seal {
    type Output;
    fn transpose(self) -> Self::Output;
}

impl<S, T> seal::Seal for Option<S>
where
    S: Strategy<Value = T> + 'static,
    T: Clone + fmt::Debug + 'static,
{
}

impl<S, T> Transpose for Option<S>
where
    S: Strategy<Value = T> + 'static,
    T: Clone + fmt::Debug + 'static,
{
    type Output = BoxedStrategy<Option<T>>;
    fn transpose(self) -> Self::Output {
        match self {
            None => Just(None).boxed(),
            Some(s) => s.prop_map(Some).boxed(),
        }
    }
}
