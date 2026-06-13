//! Self-contained pages of content.

use anyhow::{Context, Result};
use lattice::{IntoCorrect, Site, So};
use proptest::prelude::*;
use proptest_ext::transpose::Transpose;
use serde::Serialize;
use std::path::Path;
use time::{Date, UtcDateTime, serde::format_description};
use tokio::{fs::File, io::AsyncWriteExt};
use toml;

use crate::{
    section::StrategyBuilder as SectionStrategyBuilder,
    typst::file::{StrategyBuilder as TypstStrategyBuilder, TypstSourceFile},
};

/// A page's title.
// TODO replace strings with COW semantics to make cloneing cheap

#[derive(Debug, Clone, Serialize)]
pub struct Title(String);

impl Title {
    /// Strategy generating valid [`Title`]s.
    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9 ]*".prop_map(Title)
    }
}

format_description!(date_common_format, Date, "[year]-[month]-[day]");

/// A date that serializes using the common format `YYYY-MM-DD`.
#[derive(Debug, Clone, Serialize)]
struct IsoDate(#[serde(with = "date_common_format")] Date);

impl From<Date> for IsoDate {
    fn from(value: Date) -> Self {
        Self(value)
    }
}

/// A page's frontmatter.
#[derive(Debug, Serialize, Clone)]
pub struct Frontmatter {
    #[serde(skip_serializing_if = "So::is_vacant")]
    publication: So<IsoDate, String>,
    #[serde(skip_serializing_if = "So::is_vacant")]
    title: So<Title, String>,

    #[serde(skip_serializing)]
    filestem: String,
    #[serde(skip_serializing)]
    fileext: String,
}

impl Frontmatter {
    /// Create a new [`FrontmatterStrategyBuilder`] to configure the generation of [`Frontmatter`]s.
    pub fn builder() -> FrontmatterStrategyBuilder {
        FrontmatterStrategyBuilder::valid()
    }

    /// Write this [`Frontmatter`] to the specified `dir`ectory.
    ///
    /// Note that `dir` does _not_ indicate the output file's name!
    /// The output file's name is prescribed by this `Frontmatter`.
    ///
    /// # Errors
    ///
    /// Returns an error in case file system access fails.
    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        let path = dir.join(&self.filestem).with_extension(&self.fileext);
        let toml = self.try_to_toml()?;

        // No need to buffer, as we do a single write only
        let mut file = File::create(&path).await?;
        file.write_all(toml.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    /// Serialize this [`Frontmatter`] to `toml`.
    ///
    /// # Errors
    ///
    /// Returns an error in case serialization is not possible.
    pub fn try_to_toml(&self) -> Result<String> {
        let toml = toml::to_string(&self)
            .with_context(|| "while serializing a page's frontmatter to toml")?;
        Ok(toml)
    }
}

/// Builder to configure [`Strategy`]s generating [`Frontmatter`]s.
pub struct FrontmatterStrategyBuilder {
    publication: BoxedStrategy<So<IsoDate, String>>,
    title: BoxedStrategy<So<Title, String>>,

    filestem: BoxedStrategy<String>,
    fileext: BoxedStrategy<String>,
}

impl FrontmatterStrategyBuilder {
    /// Initialize generation of valid [`Frontmatter`]s.
    pub fn valid() -> Self {
        Self {
            publication: (UtcDateTime::MIN.unix_timestamp()..=UtcDateTime::MAX.unix_timestamp())
                .prop_map(|timestamp| {
                    UtcDateTime::from_unix_timestamp(timestamp)
                        .expect("timestamp in [MIN, MAX] should be valid")
                        .date()
                        .into()
                })
                .into_correct()
                .boxed(),
            title: Title::prop_valid().into_correct().boxed(),
            filestem: "page".boxed(),
            fileext: "toml".boxed(),
        }
    }

    /// Set the generated [`Frontmatter`]s' date.
    pub fn with_publication(&mut self, publication: BoxedStrategy<So<Date, String>>) -> &mut Self {
        self.publication = publication
            .prop_map(|d| d.map(|o| o.map_correct(IsoDate::from)))
            .boxed();
        self
    }

    /// Disable generation of publication dates altogether.
    pub fn without_publication(&mut self) -> &mut Self {
        self.publication = Site::prop_vacant().boxed();
        self
    }

    /// Set the generated [`Frontmatter`]'s title.
    pub fn with_title(&mut self, title: BoxedStrategy<So<Title, String>>) -> &mut Self {
        self.title = title;
        self
    }

    /// Disable generation of titles altogether.
    pub fn without_title(&mut self) -> &mut Self {
        self.title = Site::prop_vacant().boxed();
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

    /// Create a new [`Strategy`] generating [`Frontmatter`]s as configured.
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

/// A self-contained page of content.
#[derive(Debug, Clone)]
pub struct Page {
    frontmatter: Option<Frontmatter>,
    content: Option<TypstSourceFile>,
}

impl Page {
    /// Create a new [`PageStrategyBuilder`] to configure the generation of [`Page`]s.
    pub fn builder() -> PageStrategyBuilder {
        PageStrategyBuilder::valid()
    }

    /// Write this [`Page`] to the specified `dir`ectory.
    ///
    /// # Errors
    ///
    /// Returns an error in case file system access or writing of subcomponents fails.
    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        if let Some(frontmatter) = self.frontmatter.as_ref() {
            let () = frontmatter.try_write_to_dir(dir).await?;
        }

        if let Some(content) = self.content.as_ref() {
            let () = content.try_write_to_dir(dir).await?;
        }

        Ok(())
    }
}

/// Builder to configure the generation of [`Page`]s.
pub struct PageStrategyBuilder {
    frontmatter: Option<FrontmatterStrategyBuilder>,
    content: Option<TypstStrategyBuilder>,
}

impl PageStrategyBuilder {
    /// Initialize generation of valid [`Page`]s.
    pub fn valid() -> Self {
        Self {
            frontmatter: Some(FrontmatterStrategyBuilder::valid()),
            content: Some(TypstStrategyBuilder::empty()),
        }
    }

    /// Wrap [`Page`]s as configured by this [`PageStrategyBuilder`] in single-paged [`Section`]s.
    pub fn wrap_in_section(self) -> SectionStrategyBuilder {
        SectionStrategyBuilder::wrap(self)
    }

    /// Get access to this [`Page`]'s frontmatter configuration, if any.
    pub fn frontmatter(&self) -> Option<&FrontmatterStrategyBuilder> {
        self.frontmatter.as_ref()
    }

    /// Get mutable access to this [`Page`]'s frontmatter configuration, if any.
    pub fn frontmatter_mut(&mut self) -> Option<&mut FrontmatterStrategyBuilder> {
        self.frontmatter.as_mut()
    }

    /// Get access to this [`Page`]'s typst content, if any.
    pub fn content(&self) -> Option<&TypstStrategyBuilder> {
        self.content.as_ref()
    }

    /// Get mutable access to this [`Page`]'s typst content, if any.
    pub fn content_mut(&mut self) -> Option<&mut TypstStrategyBuilder> {
        self.content.as_mut()
    }

    /// Create a new [`Strategy`] generating [`Page`]'s as configured.
    pub fn build(&self) -> impl Strategy<Value = Page> + use<> {
        (
            self.frontmatter
                .as_ref()
                .map(FrontmatterStrategyBuilder::build)
                .transpose(),
            self.content
                .as_ref()
                .map(TypstStrategyBuilder::build)
                .transpose(),
        )
            .prop_map(|(frontmatter, content)| Page {
                frontmatter,
                content,
            })
    }
}

// TODO frontmatters should actually not contain the filestem/-ext. Instead, the wrapping page should store this information and pass a self-contained file-path to `try_write_to_dir`.
