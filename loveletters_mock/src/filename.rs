//! Filenames.

use proptest::prelude::*;

/// A filename given by a stem and an extension.
#[derive(Debug, Clone)]
pub struct Filename {
    stem: String,
    ext: String,
}

impl Filename {
    /// This [`Filename`]'s stem component.
    pub fn stem(&self) -> &str {
        &self.stem
    }

    /// This [`Filename`]'s extension component.
    pub fn ext(&self) -> &str {
        &self.ext
    }
}

/// Builder to configure the generation of [`Filename`]s.
pub struct StrategyBuilder {
    stem: BoxedStrategy<String>,
    ext: BoxedStrategy<String>,
}
impl StrategyBuilder {
    /// Initialize generation of [`Filename`]s.
    ///
    /// The generated [`Filename`]s feature a stem drawn from `stem` and an extension drawn from `ext`.
    pub fn new(
        stem: impl Strategy<Value = String> + 'static,
        ext: impl Strategy<Value = String> + 'static,
    ) -> Self {
        Self {
            stem: stem.boxed(),
            ext: ext.boxed(),
        }
    }

    /// Set the generated filename's stem component.
    pub fn with_stem(&mut self, stem: impl Strategy<Value = String> + 'static) -> &mut Self {
        self.stem = stem.boxed();
        self
    }

    /// Set the generated filename's extension component.
    pub fn with_ext(&mut self, ext: impl Strategy<Value = String> + 'static) -> &mut Self {
        self.ext = ext.boxed();
        self
    }

    /// Create a new [`Strategy`] generating [`Filename`]s as configured.
    pub fn build(&self) -> impl Strategy<Value = Filename> + use<> {
        let Self { stem, ext } = self;

        (stem.clone(), ext.clone()).prop_map(|(stem, ext)| Filename { stem, ext })
    }
}
