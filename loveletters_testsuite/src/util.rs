use proptest::prelude::*;

/// Wrap the value returned by the specified strategy `t` in an `Option`
/// shrinking towards `Some(_)`.
///
/// Note that shrinking towards `Some(_)` is the opposite of the default
/// shrinking behavior for `Option<T>`: it shrinks towards `None` by default.
/// However, in some use cases `Some(_)` is considered simpler ("more valid")
/// than `None`, so this adapted shrinking behavior is needed.
pub fn towards_some<T: std::fmt::Debug + Clone>(
    t: impl Strategy<Value = T>,
) -> impl Strategy<Value = Option<T>> {
    prop_oneof![t.prop_map(Option::Some), Just(None),]
}
