use anyhow::Result;
use proptest::prelude::*;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::typst::snippet::Snippet;

// TODO make cloneing cheaper!

#[derive(Debug, Clone)]
pub struct TypstSourceFile {
    content: Vec<Snippet>,

    filestem: String,
    fileext: String,
}

impl TypstSourceFile {
    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        let path = dir.join(&self.filestem).with_extension(&self.fileext);
        let mut file = BufWriter::new(File::create(&path).await?);
        for snippet in &self.content {
            let _ = file.write(snippet.as_string().as_bytes()).await?;
        }
        file.flush().await?;
        Ok(())
    }
}

pub struct StrategyBuilder {
    content: Vec<BoxedStrategy<Snippet>>,

    filestem: BoxedStrategy<String>,
    fileext: BoxedStrategy<String>,
}

impl StrategyBuilder {
    pub fn empty() -> Self {
        Self {
            content: Vec::new(),
            filestem: "page".boxed(),
            fileext: "typ".boxed(),
        }
    }

    pub fn push_snippet(&mut self, snippet: impl Strategy<Value = Snippet> + 'static) -> &mut Self {
        self.content.push(snippet.boxed());
        self
    }

    pub fn push_snippets(&mut self, snippets: &[BoxedStrategy<Snippet>]) -> &mut Self {
        self.content.extend_from_slice(snippets);
        self
    }

    pub fn with_filestem(
        &mut self,
        filestem: impl Strategy<Value = String> + 'static,
    ) -> &mut Self {
        self.filestem = filestem.boxed();
        self
    }

    pub fn with_fileext(&mut self, fileext: impl Strategy<Value = String> + 'static) -> &mut Self {
        self.fileext = fileext.boxed();
        self
    }

    pub fn build(&self) -> impl Strategy<Value = TypstSourceFile> + use<> {
        let Self {
            content,
            filestem,
            fileext,
        } = self;
        (content.clone(), filestem.clone(), fileext.clone()).prop_map(
            |(content, filestem, fileext)| TypstSourceFile {
                content,
                filestem,
                fileext,
            },
        )
    }
}
