use typst::foundations::{Dict, IntoValue, Str, Value};

use crate::{
    config::Config,
    content::{IndexFrontmatter, LeafFrontmatter},
    frontmatter_parsing::PageWithFrontmatter,
    page::{Index, Leaf},
    section::Section,
    slug::Slug,
};

#[derive(Debug, Clone)]
pub struct GlobalContext {
    content: Dict,
    config: Config,
}

impl GlobalContext {
    pub fn new(
        content: &Section<
            PageWithFrontmatter<Index, IndexFrontmatter>,
            PageWithFrontmatter<Leaf, LeafFrontmatter>,
        >,
        config: Config,
    ) -> Self {
        Self {
            content: content.to_typst(),
            config,
        }
    }
}

impl IntoValue for GlobalContext {
    fn into_value(self) -> Value {
        let Self { content, config } = self;
        let mut d = Dict::new();
        d.insert("content".into(), content.into_value());
        d.insert("config".into(), config.to_typst().into_value());
        d.into_value()
    }
}

pub struct PageContext<'a> {
    section_path: &'a [Slug],
    page: Option<&'a Slug>,
}

impl<'a> PageContext<'a> {
    pub fn new(path: &'a [Slug], page: Option<&'a Slug>) -> Self {
        Self {
            section_path: path,
            page,
        }
    }
}

impl<'a> IntoValue for PageContext<'a> {
    fn into_value(self) -> Value {
        let mut d = Dict::new();
        let path: Vec<_> = self
            .section_path
            .iter()
            .map(|s| Value::Str(Str::from(s.as_str())))
            .collect();
        d.insert("path".into(), Value::Array(path.as_slice().into()));

        if let Some(page) = self.page {
            d.insert("page".into(), Value::Str(page.as_str().into()));
        }
        d.into_value()
    }
}
