use anyhow::Result;
use lattice::{IntoCorrect, Site, So};
use proptest::prelude::*;
use serde::Serialize;
use std::{fmt, path::Path};
use time::{Date, UtcDateTime};
use tokio::{fs::File, io::AsyncWriteExt};
use toml;

// TODO replace strings with COW semantics to make cloneing cheap

#[derive(Debug, Clone, Serialize)]
pub struct Title(String);

impl Title {
    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9 ]*".prop_map(Title)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Frontmatter {
    #[serde(skip_serializing_if = "So::is_vacant")]
    publication: So<Date, String>,
    #[serde(skip_serializing_if = "So::is_vacant")]
    title: So<Title, String>,

    #[serde(skip_serializing)]
    filestem: String,
    #[serde(skip_serializing)]
    fileext: String,
}

impl Frontmatter {
    pub fn builder() -> FrontmatterStrategyBuilder {
        FrontmatterStrategyBuilder::valid()
    }

    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        let path = dir.join(&self.filestem).with_extension(&self.fileext);
        let toml = self.try_to_toml()?;

        // No need to buffer, as we do a single write only
        let mut file = File::create(&path).await?;
        file.write_all(toml.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    pub fn try_to_toml(&self) -> Result<String> {
        let toml = toml::to_string(&self)?;
        Ok(toml)
    }
}

pub struct FrontmatterStrategyBuilder {
    publication: BoxedStrategy<So<Date, String>>,
    title: BoxedStrategy<So<Title, String>>,

    filestem: BoxedStrategy<String>,
    fileext: BoxedStrategy<String>,
}

impl FrontmatterStrategyBuilder {
    pub fn valid() -> Self {
        Self {
            publication: (UtcDateTime::MIN.unix_timestamp()..=UtcDateTime::MAX.unix_timestamp())
                .prop_map(|timestamp| {
                    UtcDateTime::from_unix_timestamp(timestamp)
                        .expect("timestamp in [MIN, MAX] should be valid")
                        .date()
                })
                .into_correct()
                .boxed(),
            title: Title::prop_valid().into_correct().boxed(),
            filestem: "page".boxed(),
            fileext: "toml".boxed(),
        }
    }

    pub fn with_publication(&mut self, publication: BoxedStrategy<So<Date, String>>) -> &mut Self {
        self.publication = publication;
        self
    }

    pub fn without_publication(&mut self) -> &mut Self {
        self.publication = Site::prop_vacant().boxed();
        self
    }

    pub fn with_title(&mut self, title: BoxedStrategy<So<Title, String>>) -> &mut Self {
        self.title = title;
        self
    }

    pub fn without_title(&mut self) -> &mut Self {
        self.title = Site::prop_vacant().boxed();
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

    pub fn build(&self) -> impl Strategy<Value = Frontmatter> + use<> {
        let Self {
            publication,
            title,
            filestem,
            fileext,
        } = self;
        (
            publication.clone(),
            title.clone(),
            filestem.clone(),
            fileext.clone(),
        )
            .prop_map(|(publication, title, filestem, fileext)| Frontmatter {
                publication,
                title,
                filestem,
                fileext,
            })
    }
}
