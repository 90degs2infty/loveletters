use std::{fs, marker::PhantomData, path::PathBuf};

use anyhow::Context;
use serde::Deserialize;
use typst::foundations::{Dict, IntoValue, Value};

use crate::{
    discovery::DiscoveredPage,
    error::{Error, Result},
    page::Mode,
    rendering::{RenderedPage, Renderer, context::PageContext},
    section::Section,
    slug::Slug,
};

// TODO Instead of having a page with frontmatter, it might be more helpful to have a page with
// (page-specific/page-local) context instead.
// Then, make sure that the project-global content tree captures the same context per page as the
// (yet to implement) convenience accessor to page-local context (the one accessible via e.g.
// `#loveletters.page` or similar).

pub struct PageWithFrontmatter<M, F> {
    // TODO drop
    slug: Slug,
    content_dir: PathBuf,
    frontmatter: F,
    m: PhantomData<M>,
}

impl<M, F> PageWithFrontmatter<M, F>
where
    M: Mode,
    F: for<'de> Deserialize<'de>,
{
    pub fn try_parse(slug: Slug, dir: PathBuf) -> Result<Self> {
        let frontmatter_file = dir.join(M::frontmatter_filename());
        let frontmatter: String =
            fs::read_to_string(&frontmatter_file).map_err(|e| Error::FileIO {
                path: Some(frontmatter_file.clone()),
                raw: e,
            })?;
        let frontmatter: F = toml::from_str(&frontmatter)
            // TODO: this context is actually redundant
            .with_context(|| "Failed to parse frontmatter.")
            .map_err(|e| Error::MalformedFrontmatter {
                location: frontmatter_file,
                raw: e,
            })?;
        Ok(Self {
            slug,
            content_dir: dir,
            frontmatter,
            m: PhantomData,
        })
    }
}

impl<M, F> PageWithFrontmatter<M, F> {
    pub fn try_render(self, renderer: &Renderer, ctx: PageContext) -> Result<RenderedPage<M>>
    where
        M: Mode,
    {
        renderer.try_render_dir(self.content_dir, ctx)
    }
}

impl<'a, M, F> IntoValue for &'a PageWithFrontmatter<M, F>
where
    for<'b> &'b F: IntoValue,
{
    fn into_value(self) -> Value {
        let PageWithFrontmatter {
            slug: _,
            content_dir: _,
            frontmatter,
            m: _,
        } = self;

        let mut d = Dict::new();
        d.insert("frontmatter".into(), frontmatter.into_value());
        Value::Dict(d)
    }
}

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn try_parse<MIndex, MLeaf, FIndex, FLeaf>(
        &self,
        section: Section<DiscoveredPage<MIndex>, DiscoveredPage<MLeaf>>,
    ) -> Result<Section<PageWithFrontmatter<MIndex, FIndex>, PageWithFrontmatter<MLeaf, FLeaf>>>
    where
        MIndex: Mode,
        MLeaf: Mode,
        FIndex: for<'de> Deserialize<'de>,
        FLeaf: for<'de> Deserialize<'de>,
    {
        section.try_map(
            DiscoveredPage::<MIndex>::try_parse::<FIndex>,
            DiscoveredPage::<MLeaf>::try_parse::<FLeaf>,
        )
    }
}
