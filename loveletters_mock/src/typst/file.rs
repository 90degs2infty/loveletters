//! Self-contained typst source files.

use anyhow::{Context, Result};
use proptest::prelude::*;
use std::{path::Path, sync::Arc};
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::typst::snippet::Snippet;

/// A self-contained typst source file.
#[derive(Debug, Clone)]
pub struct TypstSourceFile {
    content: Arc<Vec<Snippet>>,
}

impl TypstSourceFile {
    /// Try to write this [`TypstSourceFile`] to the specified `path`.
    ///
    /// # Errors
    ///
    /// Returns an error in case file system access fails.
    pub async fn try_write_to(&self, path: &Path) -> Result<()> {
        let mut file = BufWriter::new(File::create(&path).await.with_context(|| {
            format!("while creating a typst source file at '{}'", path.display())
        })?);
        for snippet in self.content.iter() {
            let _ = file
                .write(snippet.as_string().as_bytes())
                .await
                .with_context(|| {
                    format!(
                        "while writing the snippet '{}' to the typst source file at '{}'",
                        snippet.as_str(),
                        path.display()
                    )
                })?;
        }
        file.flush().await.with_context(|| {
            format!(
                "while flushing the typst source file at '{}'",
                path.display()
            )
        })?;
        Ok(())
    }
}

/// Builder to configure [`Strategy`]s generating [`TypstSourceFile`]s.
pub struct StrategyBuilder {
    content: Arc<Vec<BoxedStrategy<Snippet>>>,
}

impl StrategyBuilder {
    /// Initialize generation of empty source file.
    pub fn empty() -> Self {
        Self {
            content: Arc::new(Vec::new()),
        }
    }

    /// Append the specified snippet to the generated source files.
    pub fn push_snippet(&mut self, snippet: impl Strategy<Value = Snippet> + 'static) -> &mut Self {
        Arc::make_mut(&mut self.content).push(snippet.boxed());
        self
    }

    /// Append the specified snippets to the generated source files.
    pub fn push_snippets(&mut self, snippets: &[BoxedStrategy<Snippet>]) -> &mut Self {
        Arc::make_mut(&mut self.content).extend_from_slice(snippets);
        self
    }

    /// Create a new [`Strategy`] as configured in this builder.
    pub fn build(&self) -> impl Strategy<Value = TypstSourceFile> + use<> {
        let Self { content } = self;
        (content.clone()).prop_map(|content| TypstSourceFile {
            content: Arc::new(content),
        })
    }
}
