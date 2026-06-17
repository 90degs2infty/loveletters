//! Self-contained pages of content.

use anyhow::{Context, Result, bail};
use lattice::{IntoCorrect, Site, So};
use proptest::prelude::*;
use proptest_ext::transpose::Transpose;
use serde::Serialize;
use std::{ffi::OsStr, path::Path};
use time::{Date, UtcDateTime, serde::format_description};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_stream::StreamExt;
use toml;
use walkdir::WalkDir;

use crate::{
    filename::{Filename, StrategyBuilder as FilenameStrategyBuilder},
    section::StrategyBuilder as SectionStrategyBuilder,
    typst::file::{StrategyBuilder as TypstStrategyBuilder, TypstSourceFile},
};

/// A page's title.
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
struct IsoDate(#[serde(serialize_with = "date_common_format::serialize")] Date);

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
}

impl Frontmatter {
    /// Create a new [`FrontmatterStrategyBuilder`] to configure the generation of [`Frontmatter`]s.
    #[must_use]
    pub fn builder() -> FrontmatterStrategyBuilder {
        FrontmatterStrategyBuilder::valid()
    }

    /// Write this [`Frontmatter`] to the specified `path`.
    ///
    /// # Errors
    ///
    /// Returns an error in case file system access fails.
    pub async fn try_write_to(&self, path: &Path) -> Result<()> {
        let toml = self
            .try_to_toml()
            .with_context(|| "while converting a content page's frontmatter to toml")?;

        // No need to buffer, as we do a single write only
        let mut file = File::create(&path).await.with_context(|| {
            format!("while creating a content page file at '{}'", path.display())
        })?;
        file.write_all(toml.as_bytes())
            .await
            .with_context(|| format!("while writing a content page to '{}'", path.display()))?;
        file.flush()
            .await
            .with_context(|| format!("while flushing the content page at '{}'", path.display()))?;
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
}

impl FrontmatterStrategyBuilder {
    /// Initialize generation of valid [`Frontmatter`]s.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
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

    /// Create a new [`Strategy`] generating [`Frontmatter`]s as configured.
    pub fn build(&self) -> impl Strategy<Value = Frontmatter> + use<> {
        let Self { publication, title } = self;
        (publication.clone(), title.clone())
            .prop_map(|(publication, title)| Frontmatter { publication, title })
    }
}

pub enum VerificationMode {
    IndexPage,
    LeafPage,
}

/// A self-contained page of content.
#[derive(Debug, Clone)]
pub struct Page {
    frontmatter: Option<Frontmatter>,
    frontmatter_filename: Filename,

    content: Option<TypstSourceFile>,
    typst_filename: Filename,
}

impl Page {
    /// Create a new [`PageStrategyBuilder`] to configure the generation of [`Page`]s.
    #[must_use]
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
            let path = dir
                .join(self.frontmatter_filename.stem())
                .with_extension(self.frontmatter_filename.ext());
            let () = frontmatter.try_write_to(&path).await.with_context(|| {
                format!(
                    "while writing a content page's frontmatter to '{}'",
                    path.display()
                )
            })?;
        }

        if let Some(content) = self.content.as_ref() {
            let path = dir
                .join(self.typst_filename.stem())
                .with_extension(self.typst_filename.ext());
            let () = content.try_write_to(&path).await.with_context(|| {
                format!(
                    "while writing a content page's typst source to '{}'",
                    path.display()
                )
            })?;
        }

        Ok(())
    }

    pub async fn verify_output_bundle(&self, dir: &Path, mode: VerificationMode) -> Result<()> {
        // Let's first ensure that all children we see are actually expected.
        //
        // This check has to be skipped when in mode "index page", as a section's index page is placed next to the section's subsection directories.
        if matches!(mode, VerificationMode::LeafPage) {
            let mut children = tokio_stream::iter(WalkDir::new(dir).min_depth(1).max_depth(1));

            while let Some(entry) = children.next().await {
                let entry = entry.with_context(|| {
                    format!(
                        "while enumerating children of '{}
        '",
                        dir.display()
                    )
                })?;
                let suffix = entry
                    .path()
                    .file_name()
                    .and_then(OsStr::to_str)
                    .with_context(|| {
                        format!(
                            "while converting the last component from '{}' to UTF-8",
                            entry.path().display()
                        )
                    })?;

                let expected = self.is_expected_filesystem_child(suffix);

                if !expected {
                    bail!(
                        "unexpected child '{}' at '{}' while checking a page's output bundle at '{}'",
                        suffix,
                        entry.path().display(),
                        dir.display()
                    )
                }
            }
        }

        // Now let's ensure all expected children are actually there (and have the right content).
        let index_path = dir.join("index").with_extension("html");
        let index_exists = fs::try_exists(&index_path).await.with_context(|| {
            format!(
                "while checking for existence of 'index.html' at '{}'",
                index_path.display()
            )
        })?;

        if !index_exists {
            bail!("'index.html' does not exist at '{}'", index_path.display())
        }

        Ok(())
    }

    pub fn is_expected_filesystem_child(&self, child: &str) -> bool {
        child == "index.html"
    }
}

/// Builder to configure the generation of [`Page`]s.
pub struct PageStrategyBuilder {
    frontmatter: Option<FrontmatterStrategyBuilder>,
    frontmatter_filename: FilenameStrategyBuilder,
    content: Option<TypstStrategyBuilder>,
    typst_filename: FilenameStrategyBuilder,
}

impl PageStrategyBuilder {
    /// Initialize generation of valid [`Page`]s.
    #[must_use]
    pub fn valid() -> Self {
        Self {
            frontmatter: Some(FrontmatterStrategyBuilder::valid()),
            frontmatter_filename: FilenameStrategyBuilder::new("page", "toml"),
            content: Some(TypstStrategyBuilder::empty()),
            typst_filename: FilenameStrategyBuilder::new("page", "typ"),
        }
    }

    /// Wrap [`Page`]s as configured by this [`PageStrategyBuilder`] in single-paged sections.
    #[must_use]
    pub fn wrap_in_section(self) -> SectionStrategyBuilder {
        SectionStrategyBuilder::wrap(self)
    }

    /// Get access to the generated [`Page`]'s frontmatter configuration, if any.
    #[must_use]
    pub fn frontmatter(&self) -> Option<&FrontmatterStrategyBuilder> {
        self.frontmatter.as_ref()
    }

    /// Get mutable access to the generated [`Page`]'s frontmatter configuration, if any.
    pub fn frontmatter_mut(&mut self) -> Option<&mut FrontmatterStrategyBuilder> {
        self.frontmatter.as_mut()
    }

    /// Get access to the generated [`Page`]'s frontmatter configuration filename.
    #[must_use]
    pub fn frontmatter_filename(&self) -> &FilenameStrategyBuilder {
        &self.frontmatter_filename
    }

    /// Get mutable access to the generated [`Page`]'s frontmatter configuration filename.
    pub fn frontmatter_filename_mut(&mut self) -> &mut FilenameStrategyBuilder {
        &mut self.frontmatter_filename
    }

    /// Get access to the generated [`Page`]'s typst content, if any.
    #[must_use]
    pub fn content(&self) -> Option<&TypstStrategyBuilder> {
        self.content.as_ref()
    }

    /// Get mutable access to the generated [`Page`]'s typst content, if any.
    pub fn content_mut(&mut self) -> Option<&mut TypstStrategyBuilder> {
        self.content.as_mut()
    }

    /// Get access to the generated [`Page`]'s typst source filename.
    #[must_use]
    pub fn typst_filename(&self) -> &FilenameStrategyBuilder {
        &self.typst_filename
    }

    /// Get mutable access to the generated [`Page`]'s typst source filename.
    pub fn typst_filename_mut(&mut self) -> &mut FilenameStrategyBuilder {
        &mut self.typst_filename
    }

    /// Create a new [`Strategy`] generating [`Page`]'s as configured.
    pub fn build(&self) -> impl Strategy<Value = Page> + use<> {
        let Self {
            frontmatter,
            frontmatter_filename,
            content,
            typst_filename,
        } = self;
        (
            frontmatter
                .as_ref()
                .map(FrontmatterStrategyBuilder::build)
                .transpose(),
            frontmatter_filename.build(),
            content
                .as_ref()
                .map(TypstStrategyBuilder::build)
                .transpose(),
            typst_filename.build(),
        )
            .prop_map(
                |(frontmatter, frontmatter_filename, content, typst_filename)| Page {
                    frontmatter,
                    frontmatter_filename,
                    content,
                    typst_filename,
                },
            )
    }
}
