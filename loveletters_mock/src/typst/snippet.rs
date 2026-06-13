//! Self-contained typst source snippets.

use proptest::prelude::*;

/// A self-contained snippet of typst source code.
#[derive(Debug, Clone)]
pub struct Snippet(String);

impl Snippet {
    /// The raw source code snippet.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The raw source code snippet.
    pub fn as_string(&self) -> &String {
        &self.0
    }
}

/// Indicator selecting the snippet to generate.
pub enum StrategyKind {
    /// `#lorem`
    Lorem,
    /// Some random self-contained snippet of text.
    RandomText,
}

impl StrategyKind {
    /// Convert this [`StrategyKind`] into a [`Strategy`] yielding [`Snippet`]s.
    pub fn into_strategy(&self) -> impl Strategy<Value = Snippet> {
        match self {
            Self::Lorem => Just(Snippet(String::from("#lorem(30)"))).boxed(),
            Self::RandomText => "[a-z]{30}".prop_map(Snippet).boxed(),
        }
    }
}
