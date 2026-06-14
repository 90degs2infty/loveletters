//! Self-contained loveletters projects.

use anyhow::{Context, Result};
use lattice::{IntoCorrect, So};
use proptest::prelude::*;
use proptest_ext::transpose::Transpose;
use serde::Serialize;
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

use crate::{
    filename::{Filename, StrategyBuilder as FilenameStrategyBuilder},
    section::{Section, StrategyBuilder as SectionStrategyBuilder},
};

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

/// A project's title.
#[derive(Debug, Serialize, Clone)]
pub struct Title(String);

impl Title {
    /// Create a new [`Strategy`] generating valid [`Title`]s.
    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9 ]*".prop_map(Title)
    }
}

/// A project's author.
#[derive(Debug, Serialize, Clone)]
pub struct Author(String);

impl Author {
    /// Create a new [`Strategy`] generating valid [`Author`]s.
    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9 ]*".prop_map(Author)
    }
}

/// A project's toplevel configuration.
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
}

impl Config {
    /// Create a new [`ConfigStrategyBuilder`] to configure the generation of [`Config`]s.
    pub fn builder() -> ConfigStrategyBuilder {
        ConfigStrategyBuilder::valid()
    }

    /// Try to write this [`Config`] to the specified `dir`ectory.
    ///
    /// Note that the output filename is _not_ determined by the specified `dir` but by this [`Config`].
    ///
    /// # Errors
    ///
    /// Returns an error in case file system access fails.
    pub async fn try_write_to(&self, path: &Path) -> Result<()> {
        let toml = self
            .try_to_toml()
            .with_context(|| "while converting a loveletters configuration to toml")?;

        // No need to buffer, as we do a single write only
        let mut file = File::create(&path).await.with_context(|| {
            format!(
                "while creating a loveletters configuration file at '{}'",
                path.display()
            )
        })?;
        file.write_all(toml.as_bytes()).await.with_context(|| {
            format!(
                "while writing a loveletters configuration to '{}'",
                path.display()
            )
        })?;
        file.flush().await.with_context(|| {
            format!(
                "while flushing a loveletters configuration to '{}'",
                path.display()
            )
        })?;
        Ok(())
    }

    /// Serialize this [`Config`] to `toml`.
    ///
    /// # Errors
    ///
    /// Returns an error in case serialization is not possible.
    pub fn try_to_toml(&self) -> Result<String> {
        let toml = toml::to_string(&self)
            .with_context(|| "while serializing a loveletters configuration to toml")?;
        Ok(toml)
    }
}

/// Builder to configure [`Strategy`]s generating [`Config`]s.
pub struct ConfigStrategyBuilder {
    title: BoxedStrategy<So<Title, String>>,
    author: BoxedStrategy<So<Author, String>>,
    root: BoxedStrategy<So<Url, String>>,

    excess: BoxedStrategy<Option<String>>,
}

impl ConfigStrategyBuilder {
    /// Initialize generation of valid [`Config`]s.
    pub fn valid() -> Self {
        Self {
            title: Title::prop_valid().into_correct().boxed(),
            author: Author::prop_valid().into_correct().boxed(),
            root: prop_valid_url().into_correct().boxed(),

            excess: Just(None).boxed(),
        }
    }

    /// Set the generated [`Config`]s' title.
    pub fn with_title(&mut self, title: BoxedStrategy<So<Title, String>>) -> &mut Self {
        self.title = title;
        self
    }

    /// Disable generation of titles altogether.
    pub fn without_title(&mut self) -> &mut Self {
        self.title = So::prop_vacant().boxed();
        self
    }

    /// Set the generated [`Config`]s' author.
    pub fn with_author(&mut self, author: BoxedStrategy<So<Author, String>>) -> &mut Self {
        self.author = author;
        self
    }

    /// Disable generation of authors altogether.
    pub fn without_author(&mut self) -> &mut Self {
        self.author = So::prop_vacant().boxed();
        self
    }

    /// Set the generated [`Config`]s' root Url.
    pub fn with_root(&mut self, root: BoxedStrategy<So<Url, String>>) -> &mut Self {
        self.root = root;
        self
    }

    /// Disable generation of roots altogether.
    pub fn without_root(&mut self) -> &mut Self {
        self.root = So::prop_vacant().boxed();
        self
    }

    /// Set the generated [`Config`]s' excess key-value pair.
    pub fn with_excess(&mut self, excess: BoxedStrategy<Option<String>>) -> &mut Self {
        self.excess = excess;
        self
    }

    /// Disable generation of excess keys altogether.
    pub fn without_excess(&mut self) -> &mut Self {
        self.excess = Just(None).boxed();
        self
    }
    /// Create a new [`Strategy`] generating [`Config`]s as configured.
    pub fn build(&self) -> impl Strategy<Value = Config> + use<> {
        let Self {
            title,
            author,
            root,
            excess,
        } = self;

        (title.clone(), author.clone(), root.clone(), excess.clone()).prop_map(
            |(title, author, root, excess)| Config {
                title,
                author,
                root,
                excess,
            },
        )
    }
}

/// A self-contained loveletters project.
#[derive(Debug, Clone)]
pub struct Project {
    config: Option<Config>,
    config_filename: Filename,

    content: Option<Section>,
    enforce_content_dir: bool,
}

impl Project {
    /// Create a new [`ProjectStrategyBuilder`] to configure the generation of [`Project`]s.
    pub fn builder() -> ProjectStrategyBuilder {
        ProjectStrategyBuilder::empty()
    }

    /// Try to write this [`Project`] to the specified `dir`ectory.
    ///
    /// # Errors
    ///
    /// Returns an error in case file system access or writing of subcomponents fails.
    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        let Self {
            config,
            config_filename,
            content,
            enforce_content_dir,
        } = self;

        if let Some(config) = config {
            let path = dir
                .join(config_filename.stem())
                .with_extension(config_filename.ext());
            config.try_write_to(&path).await.with_context(|| {
                format!(
                    "while writing a project's configuration to '{}'",
                    path.display()
                )
            })?;
        }

        let content_dir = dir.join("content");

        if content.is_some() || *enforce_content_dir {
            tokio::fs::create_dir(&content_dir).await.with_context(|| {
                format!(
                    "while creating a project's content directory at '{}'",
                    content_dir.display()
                )
            })?;
        }

        if let Some(content) = content {
            content
                .try_write_to_dir(&content_dir)
                .await
                .with_context(|| {
                    format!(
                        "while writing a project's content to directory '{}'",
                        content_dir.display()
                    )
                })?;
        }

        Ok(())
    }
}

/// Builder to configure [`Strategy`]s generating [`Project`]s.
pub struct ProjectStrategyBuilder {
    config: Option<ConfigStrategyBuilder>,
    config_filename: FilenameStrategyBuilder,

    content: Option<SectionStrategyBuilder>,
    enforce_content_dir: BoxedStrategy<bool>,
}

impl ProjectStrategyBuilder {
    /// Initialize creation of empty [`Project`]s.
    pub fn empty() -> Self {
        Self {
            config: Some(ConfigStrategyBuilder::valid()),
            config_filename: FilenameStrategyBuilder::new("loveletters", "toml"),
            content: Some(SectionStrategyBuilder::empty()),
            enforce_content_dir: Just(false).boxed(),
        }
    }

    /// Get mutable access to the generated [`Project`]s' toplevel config configuration, if any.
    pub fn config_mut(&mut self) -> Option<&mut ConfigStrategyBuilder> {
        self.config.as_mut()
    }

    /// Get access to the generated [`Project`]'s configuration filename.
    pub fn config_filename(&self) -> &FilenameStrategyBuilder {
        &self.config_filename
    }

    /// Get mutable access to the generated [`Project`]'s configuration filename.
    pub fn config_filename_mut(&mut self) -> &mut FilenameStrategyBuilder {
        &mut self.config_filename
    }

    /// Get mutable access to the generated [`Project`]s' content configuration, if any.
    pub fn content_mut(&mut self) -> Option<&mut SectionStrategyBuilder> {
        self.content.as_mut()
    }

    /// Set the generated [`Project`]s' configuration.
    pub fn with_config(&mut self, config: ConfigStrategyBuilder) -> &mut Self {
        self.config = Some(config);
        self
    }

    /// Disable generation of configurations altogether.
    pub fn without_config(&mut self) -> &mut Self {
        self.config = None;
        self
    }

    /// Set the generated [`Project`]s' content.
    pub fn with_content(&mut self, content: SectionStrategyBuilder) -> &mut Self {
        self.content = Some(content);
        self
    }

    /// Disable generation of content altogether.
    pub fn without_content(&mut self) -> &mut Self {
        self.content = None;
        self
    }

    /// Enforce the generation of content directories, irrespective of the generation of actual content.
    ///
    /// In case no content is generated but the generation of content directories is enforced, content directories remain empty.
    pub fn enforce_content_dir(&mut self) -> &mut Self {
        self.enforce_content_dir = Just(true).boxed();
        self
    }

    /// Generate content directories only when also generating actual content.
    pub fn create_content_dir_lazy(&mut self) -> &mut Self {
        self.enforce_content_dir = Just(false).boxed();
        self
    }

    /// Create a new [`Strategy`] generating [`Project`]s as configured.
    pub fn build(&self) -> impl Strategy<Value = Project> + use<> {
        let Self {
            config,
            config_filename,
            content,
            enforce_content_dir,
        } = self;

        (
            config
                .as_ref()
                .map(ConfigStrategyBuilder::build)
                .transpose(),
            config_filename.build(),
            content
                .as_ref()
                .map(SectionStrategyBuilder::build)
                .transpose(),
            enforce_content_dir.clone(),
        )
            .prop_map(
                |(config, config_filename, content, enforce_content_dir)| Project {
                    config,
                    config_filename,
                    content,
                    enforce_content_dir,
                },
            )
    }
}
