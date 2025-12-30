use typst::foundations::{Dict, IntoValue, Str, Value};

use crate::{
    content::{IndexFrontmatter, LeafFrontmatter},
    frontmatter_parsing::PageWithFrontmatter,
    page::{Index, Leaf},
    section::Section,
    slug::Slug,
};

// TODO builder pattern?
#[derive(Debug, Clone)]
pub struct GlobalContext {
    content: Dict,
}

impl GlobalContext {
    pub fn new(
        content: &Section<
            PageWithFrontmatter<Index, IndexFrontmatter>,
            PageWithFrontmatter<Leaf, LeafFrontmatter>,
        >,
    ) -> Self {
        Self {
            content: content.to_typst(),
        }
    }
}

impl IntoValue for GlobalContext {
    fn into_value(self) -> Value {
        let mut d = Dict::new();
        d.insert("content".into(), self.content.into_value());
        d.into_value()
    }
}

// TODO builder pattern?
pub struct PageContext<'a> {
    section_path: &'a [Slug],
    page: Option<&'a Slug>,
    frontmatter: Value,
}

impl<'a> PageContext<'a> {
    pub fn new(path: &'a [Slug], page: Option<&'a Slug>) -> Self {
        Self {
            section_path: path,
            page,
            frontmatter: Value::None,
        }
    }

    pub fn with_frontmatter<F: IntoValue>(&mut self, frontmatter: F) -> &mut Self {
        self.frontmatter = frontmatter.into_value();
        self
    }
}

impl<'a> IntoValue for PageContext<'a> {
    fn into_value(self) -> Value {
        let Self {
            section_path,
            page,
            frontmatter,
        } = self;
        let mut d = Dict::new();

        let path: Vec<_> = section_path
            .iter()
            .map(|s| Value::Str(Str::from(s.as_str())))
            .collect();
        d.insert("path".into(), Value::Array(path.as_slice().into()));

        if let Some(page) = page {
            d.insert("page".into(), Value::Str(page.as_str().into()));
        }

        d.insert("frontmatter".into(), frontmatter);

        d.into_value()
    }
}
