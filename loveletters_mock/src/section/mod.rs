use anyhow::Result;
use proptest::{
    collection::{hash_set, vec},
    prelude::*,
    sample::SizeRange,
};
use proptest_ext::transpose::Transpose;
use std::{
    collections::{HashMap, HashSet},
    hash,
    path::Path,
};
use tokio::fs;

use crate::page::leaf::{Page, PageStrategyBuilder};

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
            let _ = shard.push(
                elements
                    .next()
                    .expect("set should have provided enough unique elements"),
            );
        }
        shards.push(shard);
    }

    shards
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

impl Slug {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9]{1,}".prop_map(Self)
    }
}

// TODO make clone cheaper
#[derive(Debug, Clone)]
pub struct Section {
    index: Option<Page>,
    subsections: HashMap<Slug, Section>,
    pages: HashMap<Slug, Page>,

    // An excess page that is written top-level to this section's directory (and has to be ignored
    // during content discovery)
    clutter: Option<Page>,
}

impl Section {
    pub async fn try_write_to_dir(&self, dir: &Path) -> Result<()> {
        let Self {
            index,
            subsections,
            pages,
            clutter,
        } = self;

        if let Some(index) = index.as_ref() {
            let index_dir = dir.join("_index");
            fs::create_dir(&index_dir).await?;
            index.try_write_to_dir(&index_dir).await?;
        }

        // TODO write in parallel?
        for (slug, page) in pages.iter() {
            let page_dir = dir.join(slug.as_str());
            fs::create_dir(&page_dir).await?;
            page.try_write_to_dir(&page_dir).await?;
        }

        // TODO write in parallel?
        for (slug, subsection) in subsections.iter() {
            let subsection_dir = dir.join(slug.as_str());
            fs::create_dir(&subsection_dir).await?;
            Box::pin(subsection.try_write_to_dir(&subsection_dir)).await?;
        }

        if let Some(clutter) = clutter.as_ref() {
            clutter.try_write_to_dir(dir).await?;
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

pub struct StrategyBuilder {
    index: Option<PageStrategyBuilder>,
    subsections: Subsections,
    pages: Pages,
    clutter: Option<PageStrategyBuilder>,
}

impl StrategyBuilder {
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

    pub fn wrap(page: PageStrategyBuilder) -> Self {
        let mut builder = Self::empty();
        builder.push_page(page);
        builder
    }

    pub fn with_index(&mut self, page: PageStrategyBuilder) -> &mut Self {
        self.index = Some(page);
        self
    }

    pub fn without_index(&mut self) -> &mut Self {
        self.index = None;
        self
    }

    pub fn with_clutter(&mut self, clutter: PageStrategyBuilder) -> &mut Self {
        self.clutter = Some(clutter);
        self
    }

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

                // (index, subsections, pages, clutter)
                //     .prop_flat_map(|(index, subsections, pages, clutter)| {
                //         (
                //             Just(index),
                //             hash_set(Slug::prop_valid(), subsections.len() + pages.len()),
                //             Just(subsections),
                //             Just(pages),
                //             Just(clutter),
                //         )
                //     })
                //     .prop_map(|(index, slugs, subsections, pages, clutter)| {
                //         let mut slug_shards = sharden(slugs, vec![subsections.len(), pages.len()]);
                //         // Mind the reverse order
                //         let page_slugs = slug_shards
                //             .pop()
                //             .expect("slug_shards should hold exactly two items by construction");
                //         let subsection_slugs = slug_shards
                //             .pop()
                //             .expect("slug_shards should hold exactly two items by construction");

                //         let subsections = HashMap::from_iter(
                //             subsection_slugs.into_iter().zip(subsections.into_iter()),
                //         );

                //         let pages =
                //             HashMap::from_iter(page_slugs.into_iter().zip(pages.into_iter()));

                //         Section {
                //             index,
                //             subsections,
                //             pages,
                //             clutter,
                //         }
                //     })
                //     .boxed()
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

// TODO make all the builders implement clone and debug?

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

            let subsections =
                HashMap::from_iter(subsection_slugs.into_iter().zip(subsections.into_iter()));

            let pages = HashMap::from_iter(page_slugs.into_iter().zip(pages.into_iter()));

            Section {
                index,
                subsections,
                pages,
                clutter,
            }
        })
        .boxed()
}

// TODO context for applications of ?
