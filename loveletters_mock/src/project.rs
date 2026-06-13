use anyhow::Result;
use lattice::{IntoCorrect, So};
use proptest::prelude::*;
use proptest_ext::transpose::Transpose;
use serde::Serialize;
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

use crate::section::{Section, StrategyBuilder as SectionStrategyBuilder};

fn prop_valid_url() -> impl Strategy<Value = Url> {
    // at least foo.bar with optional leading subdomains and optional trailing paths (paths can end
    // on / or not)
    "https?://([a-z0-9]+\\.)+[a-z]+(/[a-z0-9]+)*/?".prop_map(|raw| {
        Url::parse(&raw).expect(&format!(
            "regex should generate valid Url, but yielded {}",
            &raw
        ))
    })
}

// TODO: make cloneing more performant
#[derive(Debug, Serialize, Clone)]
pub struct Title(String);

impl Title {
    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9 ]*".prop_map(Title)
    }
}

// TODO: make cloneing more performant
#[derive(Debug, Serialize, Clone)]
pub struct Author(String);

impl Author {
    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9 ]*".prop_map(Author)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Config {
    #[serde(skip_serializing_if = "So::is_vacant")]
    title: So<Title, String>,
    #[serde(skip_serializing_if = "So::is_vacant")]
    author: So<Author, String>,
    #[serde(skip_serializing_if = "So::is_vacant")]
    root: So<Url, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    excess: Option<String>,

    #[serde(skip_serializing)]
    filestem: String,
    #[serde(skip_serializing)]
    fileext: String,
}

impl Config {
    pub fn builder() -> ConfigStrategyBuilder {
        ConfigStrategyBuilder::valid()
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

pub struct ConfigStrategyBuilder {
    title: BoxedStrategy<So<Title, String>>,
    author: BoxedStrategy<So<Author, String>>,
    root: BoxedStrategy<So<Url, String>>,

    excess: BoxedStrategy<Option<String>>,

    filestem: BoxedStrategy<String>,
    fileext: BoxedStrategy<String>,
}

impl ConfigStrategyBuilder {
    pub fn valid() -> Self {
        Self {
            title: Title::prop_valid().into_correct().boxed(),
            author: Author::prop_valid().into_correct().boxed(),
            root: prop_valid_url().into_correct().boxed(),

            excess: Just(None).boxed(),

            filestem: Just("loveletters".to_owned()).boxed(),
            fileext: Just("toml".to_owned()).boxed(),
        }
    }

    pub fn with_title(&mut self, title: BoxedStrategy<So<Title, String>>) -> &mut Self {
        self.title = title;
        self
    }

    pub fn without_title(&mut self) -> &mut Self {
        self.title = So::prop_vacant().boxed();
        self
    }

    pub fn with_author(&mut self, author: BoxedStrategy<So<Author, String>>) -> &mut Self {
        self.author = author;
        self
    }

    pub fn without_author(&mut self) -> &mut Self {
        self.author = So::prop_vacant().boxed();
        self
    }

    pub fn with_root(&mut self, root: BoxedStrategy<So<Url, String>>) -> &mut Self {
        self.root = root;
        self
    }

    pub fn without_root(&mut self) -> &mut Self {
        self.root = So::prop_vacant().boxed();
        self
    }

    pub fn with_excess(&mut self, excess: BoxedStrategy<Option<String>>) -> &mut Self {
        self.excess = excess;
        self
    }

    pub fn without_excess(&mut self) -> &mut Self {
        self.excess = Just(None).boxed();
        self
    }

    pub fn with_filestem(&mut self, filestem: BoxedStrategy<String>) -> &mut Self {
        self.filestem = filestem;
        self
    }

    pub fn with_fileext(&mut self, fileext: BoxedStrategy<String>) -> &mut Self {
        self.fileext = fileext;
        self
    }

    pub fn build(&self) -> impl Strategy<Value = Config> + use<> {
        let Self {
            title,
            author,
            root,
            excess,
            filestem,
            fileext,
        } = self;

        (
            title.clone(),
            author.clone(),
            root.clone(),
            excess.clone(),
            filestem.clone(),
            fileext.clone(),
        )
            .prop_map(|(title, author, root, excess, filestem, fileext)| Config {
                title,
                author,
                root,
                excess,
                filestem,
                fileext,
            })
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    config: Option<Config>,
    content: Option<Section>,

    enforce_content_dir: bool,
}

impl Project {
    pub fn builder() -> ProjectStrategyBuilder {
        ProjectStrategyBuilder::empty()
    }

    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        let Self {
            config,
            content,
            enforce_content_dir,
        } = self;

        if let Some(config) = config {
            config.try_write_to_dir(dir).await?;
        }

        let content_dir = dir.join("content");

        if content.is_some() || *enforce_content_dir {
            tokio::fs::create_dir(&content_dir).await?;
        }

        if let Some(content) = content {
            content.try_write_to_dir(&content_dir).await?;
        }

        Ok(())
    }
}

pub struct ProjectStrategyBuilder {
    config: Option<ConfigStrategyBuilder>,
    content: Option<SectionStrategyBuilder>,

    enforce_content_dir: BoxedStrategy<bool>,
}

impl ProjectStrategyBuilder {
    pub fn empty() -> Self {
        Self {
            config: Some(ConfigStrategyBuilder::valid()),
            content: Some(SectionStrategyBuilder::empty()),
            enforce_content_dir: Just(false).boxed(),
        }
    }

    pub fn config_mut(&mut self) -> Option<&mut ConfigStrategyBuilder> {
        self.config.as_mut()
    }

    pub fn content_mut(&mut self) -> Option<&mut SectionStrategyBuilder> {
        self.content.as_mut()
    }

    pub fn with_config(&mut self, config: ConfigStrategyBuilder) -> &mut Self {
        self.config = Some(config);
        self
    }

    pub fn without_config(&mut self) -> &mut Self {
        self.config = None;
        self
    }

    pub fn with_content(&mut self, content: SectionStrategyBuilder) -> &mut Self {
        self.content = Some(content);
        self
    }

    pub fn without_content(&mut self) -> &mut Self {
        self.content = None;
        self
    }

    pub fn enforce_content_dir(&mut self) -> &mut Self {
        self.enforce_content_dir = Just(true).boxed();
        self
    }

    pub fn create_content_dir_lazy(&mut self) -> &mut Self {
        self.enforce_content_dir = Just(false).boxed();
        self
    }

    pub fn build(&self) -> impl Strategy<Value = Project> + use<> {
        let Self {
            config,
            content,
            enforce_content_dir,
        } = self;

        (
            config
                .as_ref()
                .map(ConfigStrategyBuilder::build)
                .transpose(),
            content
                .as_ref()
                .map(SectionStrategyBuilder::build)
                .transpose(),
            enforce_content_dir.clone(),
        )
            .prop_map(|(config, content, enforce_content_dir)| Project {
                config,
                content,
                enforce_content_dir,
            })
    }
}
