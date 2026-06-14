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

    filestem: String,
    fileext: String,
}

impl TypstSourceFile {
    /// Try to write this [`TypstSourceFile`] to the specified directory.
    ///
    /// Note that the output filename is _not_ determined by the specified `dir` but by this [`TypstSourceFile`].
    ///
    /// # Errors
    ///
    /// Returns an error in case file system access fails.
    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        let path = dir.join(&self.filestem).with_extension(&self.fileext);
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

    filestem: BoxedStrategy<String>,
    fileext: BoxedStrategy<String>,
}

impl StrategyBuilder {
    /// Initialize generation of empty source file.
    pub fn empty() -> Self {
        Self {
            content: Arc::new(Vec::new()),
            filestem: "page".boxed(),
            fileext: "typ".boxed(),
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

    /// Set the generated source files' filename stem component.
    pub fn with_filestem(
        &mut self,
        filestem: impl Strategy<Value = String> + 'static,
    ) -> &mut Self {
        self.filestem = filestem.boxed();
        self
    }

    /// Set the generated source files' filename extension component.
    pub fn with_fileext(&mut self, fileext: impl Strategy<Value = String> + 'static) -> &mut Self {
        self.fileext = fileext.boxed();
        self
    }

    /// Create a new [`Strategy`] as configured in this builder.
    pub fn build(&self) -> impl Strategy<Value = TypstSourceFile> + use<> {
        let Self {
            content,
            filestem,
            fileext,
        } = self;
        (content.clone(), filestem.clone(), fileext.clone()).prop_map(
            |(content, filestem, fileext)| TypstSourceFile {
                content: Arc::new(content),
                filestem,
                fileext,
            },
        )
    }
}
