//! Self-contained sections of content.

use anyhow::{Context, Result, bail};
use proptest::{
    collection::{hash_set, vec},
    prelude::*,
    sample::SizeRange,
};
use proptest_ext::transpose::Transpose;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    ffi::OsStr,
    hash::{self},
    path::Path,
    sync::Arc,
};
use tokio::fs;
use tokio_stream::StreamExt;
use walkdir::WalkDir;

use crate::page::{Page, PageStrategyBuilder, VerificationMode};

// set's size has to match the sum of sizes - this is not checked!
//
// The returned vecs are guaranteed to hold globally unique values. The individual shards are
// returned as Vecs for performance reasons, collect them into a HashSet if required.
fn sharden<T: Eq + hash::Hash>(mut set: HashSet<T>, sizes: Vec<usize>) -> Vec<Vec<T>> {
    let mut shards = Vec::with_capacity(sizes.len());

    let mut elements = set.drain();
    for s in sizes {
        let mut shard = Vec::with_capacity(s);

        for _ in 0..s {
            shard.push(
                elements
                    .next()
                    .expect("set should have provided enough unique elements"),
            );
        }
        shards.push(shard);
    }

    shards
}

/// A key indicating the location of either a sub-section or leaf page.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

impl Slug {
    /// Get this key as `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Strategy generating valid [`Slug`]s.
    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9]{1,}".prop_map(Self)
    }
}

impl Borrow<str> for Slug {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A self-contained section of content potentially including sub-sections and leaf pages.
#[derive(Debug, Clone)]
pub struct Section {
    index: Option<Page>,
    subsections: Arc<HashMap<Slug, Section>>,
    pages: Arc<HashMap<Slug, Page>>,

    // An excess page that is written top-level to this section's directory (and has to be ignored
    // during content discovery)
    clutter: Option<Page>,
}

impl Section {
    /// Try to write this section into the specified `dir`.
    ///
    /// # Errors
    ///
    /// In case file system access or writing of the contained content fails.
    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        let Self {
            index,
            subsections,
            pages,
            clutter,
        } = self;

        if let Some(index) = index.as_ref() {
            let index_dir = dir.join("_index");
            fs::create_dir(&index_dir).await.with_context(|| {
                format!(
                    "while creating a section's index page directory at '{}'",
                    index_dir.display()
                )
            })?;
            index.try_write_to_dir(&index_dir).await.with_context(|| {
                format!(
                    "while writing a section's index page to directory '{}'",
                    index_dir.display()
                )
            })?;
        }

        // ISSUE pages could be written concurrently to maybe speed up write out
        for (slug, page) in pages.iter() {
            let page_dir = dir.join(slug.as_str());
            fs::create_dir(&page_dir).await.with_context(|| {
                format!(
                    "while creating a leaf page's directory at '{}', slug '{}'",
                    page_dir.display(),
                    slug.as_str()
                )
            })?;
            page.try_write_to_dir(&page_dir).await.with_context(|| {
                format!(
                    "while writing a leaf page to directory '{}', slug '{}'",
                    slug.as_str(),
                    page_dir.display()
                )
            })?;
        }

        // ISSUE pages could be written concurrently to maybe speed up write out
        for (slug, subsection) in subsections.iter() {
            let subsection_dir = dir.join(slug.as_str());
            fs::create_dir(&subsection_dir).await.with_context(|| {
                format!(
                    "while creating a subsection's directory at '{}', slug '{}'",
                    subsection_dir.display(),
                    slug.as_str()
                )
            })?;
            Box::pin(subsection.try_write_to_dir(&subsection_dir))
                .await
                .with_context(|| {
                    format!(
                        "while writing a subsection to directory '{}', slug '{}'",
                        slug.as_str(),
                        subsection_dir.display()
                    )
                })?;
        }

        if let Some(clutter) = clutter.as_ref() {
            clutter
                .try_write_to_dir(dir)
                .await
                .with_context(|| format!("while writing a clutter page to '{}'", dir.display()))?;
        }

        Ok(())
    }

    pub async fn verify_output_bundle(&self, dir: &Path) -> Result<()> {
        let index = self
            .index
            .as_ref()
            .expect("verify_output_bundle should only be called on valid sections");

        // Let's first ensure that all children we see are actually expected.
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

            let expected = index.is_expected_filesystem_child(suffix)
                || self.pages.contains_key(suffix)
                || self.subsections.contains_key(suffix);

            if !expected {
                bail!(
                    "unexpected child '{}' at '{}' while checking a section's output bundle at '{}'",
                    suffix,
                    entry.path().display(),
                    dir.display()
                )
            }
        }

        // Now let's ensure all expected children are actually there (and have the right content).
        index
            .verify_output_bundle(dir, VerificationMode::IndexPage)
            .await
            .with_context(|| {
                format!(
                    "while verifying the output bundle in '{}' for a section's index page",
                    dir.display()
                )
            })?;

        for (slug, page) in self.pages.iter() {
            let child = dir.join(slug.as_str());
            page.verify_output_bundle(&child, VerificationMode::LeafPage)
                .await
                .with_context(|| {
                    format!(
                        "while verifying the output bundle in '{}' for the leaf page at '{}'",
                        child.display(),
                        slug.as_str()
                    )
                })?;
        }

        for (slug, subsec) in self.subsections.iter() {
            let child = dir.join(slug.as_str());
            Box::pin(subsec.verify_output_bundle(&child))
                .await
                .with_context(|| {
                    format!(
                        "while verifying the output bundle in '{}' for the subsection at '{}'",
                        child.display(),
                        slug.as_str()
                    )
                })?;
        }

        Ok(())
    }
}

enum Pages {
    OneByOne {
        pages: Vec<PageStrategyBuilder>,
    },
    Repeat {
        page: PageStrategyBuilder,
        num: SizeRange,
    },
}

enum Subsections {
    OneByOne {
        sections: Vec<StrategyBuilder>,
    },
    Recurse {
        // maximum level of nesting of sections
        max_depth: u32,
        // desired number of sections contained in the outermost tree of sections
        desired_num_sections: u32,
        // maximum number of direct subsections contained in any section
        max_branching: u32,
    },
}

/// Builder to configure [`Strategy`]s generating [`Section`]s.
pub struct StrategyBuilder {
    index: Option<PageStrategyBuilder>,
    subsections: Subsections,
    pages: Pages,
    clutter: Option<PageStrategyBuilder>,
}

impl StrategyBuilder {
    /// Initalize the generation of empty [`Section`]s.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            index: Some(PageStrategyBuilder::valid()),
            subsections: Subsections::OneByOne {
                sections: Vec::new(),
            },
            pages: Pages::OneByOne { pages: Vec::new() },
            clutter: None,
        }
    }

    /// Wrap pages generated by `page` as single-page [`Section`].
    #[must_use]
    pub fn wrap(page: PageStrategyBuilder) -> Self {
        let mut builder = Self::empty();
        builder.push_page(page);
        builder
    }

    /// Set the index pages to generate.
    pub fn with_index(&mut self, page: PageStrategyBuilder) -> &mut Self {
        self.index = Some(page);
        self
    }

    /// Do not generate any index pages at all.
    ///
    /// Also see [`Self::with_index`].
    pub fn without_index(&mut self) -> &mut Self {
        self.index = None;
        self
    }

    /// Generate some clutter page that will be written to the generated section's toplevel
    /// directory.
    ///
    /// Clutter should not be picked up by `loveletters`' internal processing.
    pub fn with_clutter(&mut self, clutter: PageStrategyBuilder) -> &mut Self {
        self.clutter = Some(clutter);
        self
    }

    /// Do not generate any clutter.
    ///
    /// Also see [`Self::with_clutter`].
    pub fn without_clutter(&mut self) -> &mut Self {
        self.clutter = None;
        self
    }

    /// Append the specified page to this section.
    ///
    /// [`Self::push_page`] and [`Self::draw_pages`] are mutually exclusive: note that in case this
    /// builder was configured to draw pages according to [`Self::draw_pages`] previously, said
    /// configuration is discarded.
    pub fn push_page(&mut self, page: PageStrategyBuilder) -> &mut Self {
        match &mut self.pages {
            Pages::OneByOne { pages } => {
                pages.push(page);
            }
            Pages::Repeat { page: _, num: _ } => {
                self.pages = Pages::OneByOne { pages: vec![page] };
            }
        }
        self
    }

    /// Draw `num` pages from `page`.
    ///
    /// [`Self::draw_pages`] and [`Self::push_page`] are mutually exclusive: note that in case this
    /// builder was configured to generate specific pages according to [`Self::push_page`] previously,
    /// said configuration is discarded.
    /// I.e. on return, this [`StrategyBuilder`] will only generate pages drawn from `page`.
    pub fn draw_pages<R: Into<SizeRange>>(
        &mut self,
        page: PageStrategyBuilder,
        num: R,
    ) -> &mut Self {
        self.pages = Pages::Repeat {
            page,
            num: num.into(),
        };
        self
    }

    /// Append the specified subsection to this section.
    ///
    /// [`Self::push_subsection`] and [`Self::recurse`] are mutually exclusive: note that in case this
    /// builder was configured to draw subsections according to [`Self::recurse`] previously, said
    /// configuration is discarded.
    pub fn push_subsection(&mut self, subsection: Self) -> &mut Self {
        match &mut self.subsections {
            Subsections::OneByOne { sections } => {
                sections.push(subsection);
            }
            Subsections::Recurse {
                max_depth: _,
                desired_num_sections: _,
                max_branching: _,
            } => {
                self.subsections = Subsections::OneByOne {
                    sections: vec![subsection],
                };
            }
        }
        self
    }

    /// Generate subsections by recursing this [`StrategyBuilder`].
    ///
    /// [`Self::push_subsection`] and [`Self::recurse`] are mutually exclusive: note that in case this
    /// builder was configured to draw specific subsections according to [`Self::push_subsection`]
    /// previously, said configuration is discarded.
    pub fn recurse(
        &mut self,
        // maximum level of nesting of sections
        max_depth: u32,
        // desired number of sections contained in the outermost tree of sections
        desired_num_sections: u32,
        // maximum number of direct subsections contained in any section
        max_branching: u32,
    ) -> &mut Self {
        self.subsections = Subsections::Recurse {
            max_depth,
            desired_num_sections,
            max_branching,
        };
        self
    }

    /// Create a new [`Strategy`] generating [`Section`]s as configured.
    pub fn build(&self) -> impl Strategy<Value = Section> + use<> {
        let Self {
            index,
            subsections,
            pages,
            clutter,
        } = self;

        let index = index.as_ref().map(PageStrategyBuilder::build).transpose();

        let pages = match pages {
            Pages::OneByOne { pages } => pages
                .iter()
                .map(PageStrategyBuilder::build)
                .collect::<Vec<_>>()
                .boxed(),
            Pages::Repeat { page, num } => vec(page.build(), num.clone()).boxed(),
        };

        let clutter = clutter.as_ref().map(PageStrategyBuilder::build).transpose();

        match subsections {
            Subsections::OneByOne {
                sections: subsections,
            } => {
                let subsections = subsections
                    .iter()
                    .map(StrategyBuilder::build)
                    .collect::<Vec<_>>()
                    .boxed();

                prop_wrap_in_section(index, subsections, pages, clutter)
            }
            Subsections::Recurse {
                max_depth,
                desired_num_sections,
                max_branching,
            } => {
                let leaf_section = prop_wrap_in_section(
                    index.clone(),
                    Just(Vec::new()).boxed(),
                    pages.clone(),
                    clutter.clone(),
                );

                let max_branching = *max_branching;

                leaf_section
                    .prop_recursive(
                        *max_depth,
                        *desired_num_sections,
                        max_branching,
                        move |element| {
                            let subsections = vec(element, 0..=(max_branching as usize)).boxed();

                            prop_wrap_in_section(
                                index.clone(),
                                subsections,
                                pages.clone(),
                                clutter.clone(),
                            )
                        },
                    )
                    .boxed()
            }
        }
    }
}

impl From<PageStrategyBuilder> for StrategyBuilder {
    fn from(value: PageStrategyBuilder) -> Self {
        Self::wrap(value)
    }
}

fn prop_wrap_in_section(
    index: BoxedStrategy<Option<Page>>,
    subsections: BoxedStrategy<Vec<Section>>,
    pages: BoxedStrategy<Vec<Page>>,
    clutter: BoxedStrategy<Option<Page>>,
) -> BoxedStrategy<Section> {
    (index, subsections, pages, clutter)
        .prop_flat_map(|(index, subsections, pages, clutter)| {
            (
                Just(index),
                hash_set(Slug::prop_valid(), subsections.len() + pages.len()),
                Just(subsections),
                Just(pages),
                Just(clutter),
            )
        })
        .prop_map(|(index, slugs, subsections, pages, clutter)| {
            let mut slug_shards = sharden(slugs, vec![subsections.len(), pages.len()]);
            // Mind the reverse order
            let page_slugs = slug_shards
                .pop()
                .expect("slug_shards should hold exactly two items by construction");
            let subsection_slugs = slug_shards
                .pop()
                .expect("slug_shards should hold exactly two items by construction");

            let subsections = subsection_slugs.into_iter().zip(subsections).collect();

            let pages = page_slugs.into_iter().zip(pages).collect();

            Section {
                index,
                subsections: Arc::new(subsections),
                pages: Arc::new(pages),
                clutter,
            }
        })
        .boxed()
}
