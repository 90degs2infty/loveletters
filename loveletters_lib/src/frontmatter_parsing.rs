use std::{fs, marker::PhantomData, path::PathBuf};

use serde::Deserialize;
use typst::foundations::{Dict, IntoValue, Value};

use crate::{
    discovery::DiscoveredPage,
    error::Result,
    page::Mode,
    rendering::{RenderedPage, context::Context},
    section::Section,
    slug::Slug,
};

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
        let frontmatter: String = fs::read_to_string(&frontmatter_file).expect(&format!(
            "Failed to open frontmatter file at '{}'",
            frontmatter_file.display()
        ));
        let frontmatter: F = toml::from_str(&frontmatter).expect(&format!(
            "Failed to parse frontmatter from file at '{}'",
            frontmatter_file.display()
        ));
        Ok(Self {
            slug,
            content_dir: dir,
            frontmatter,
            m: PhantomData,
        })
    }
}

impl<M, F> PageWithFrontmatter<M, F> {
    pub fn try_render(self, ctx: Context) -> Result<RenderedPage<M>>
    where
        M: Mode,
    {
        RenderedPage::try_render(self.slug, self.content_dir, M::typst_filename().into(), ctx)
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
