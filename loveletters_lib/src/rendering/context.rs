use typst::foundations::{Dict, IntoValue, Str, Value};

use crate::slug::Slug;

// TODO implement this properly...
// probably, it is more useful to have a global context (which captures e.g. config, the content tree etc.)
// and a page-local one (which captures front-matter, section_path, page etc.) and pass both individually
// to the rendering business logic.
// TODO frontmatter
pub struct Context<'a> {
    tree: Dict,
    section_path: &'a [Slug],
    page: Option<&'a Slug>,
}

impl<'a> Context<'a> {
    pub fn new(tree: Dict, section_path: &'a [Slug], page: Option<&'a Slug>) -> Self {
        Self {
            tree,
            section_path,
            page,
        }
    }
}

impl<'a, 'b> IntoValue for Context<'a> {
    fn into_value(self) -> Value {
        let mut d = Dict::new();
        d.insert("content".into(), Value::Dict(self.tree));
        let path: Vec<_> = self
            .section_path
            .iter()
            .map(|s| Value::Str(Str::from(s.as_str())))
            .collect();
        d.insert("path".into(), Value::Array(path.as_slice().into()));

        if let Some(page) = self.page {
            d.insert("page".into(), Value::Str(page.as_str().into()));
        }

        return Value::Dict(d);
    }
}
