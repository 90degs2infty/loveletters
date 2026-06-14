use std::{fs, path::PathBuf};

use serde::Deserialize;
use typst::foundations::{Dict, IntoValue, Value};

use crate::{
    constants::{FRONTMATTER_FILEEXT, PAGE_FILESTEM},
    discovery::DiscoveredPage,
    error::{Error, Result},
    rendering::{RenderedPage, Renderer, context::PageContext},
    section::Section,
};

// TODO Instead of having a page with frontmatter, it might be more helpful to have a page with
// (page-specific/page-local) context instead.
// Then, make sure that the project-global content tree captures the same context per page as the
// (yet to implement) convenience accessor to page-local context (the one accessible via e.g.
// `#loveletters.page` or similar).

// TODO get rid of generic parameter, use Frontmatter directly instead
pub struct PageWithFrontmatter<F> {
    content_dir: PathBuf,
    frontmatter: F,
}

impl<F> PageWithFrontmatter<F>
where
    F: for<'de> Deserialize<'de>,
{
    pub fn try_parse(dir: PathBuf) -> Result<Self> {
        let frontmatter_file = dir.join(PAGE_FILESTEM).with_extension(FRONTMATTER_FILEEXT);
        let frontmatter: String =
            fs::read_to_string(&frontmatter_file).map_err(|e| Error::FileIO {
                path: Some(frontmatter_file.clone()),
                raw: e,
            })?;
        let frontmatter = toml::from_str(&frontmatter)
            .map_err(|e| (Error::MalformedFrontmatter { raw: Box::new(e) }).at(frontmatter_file))?;
        Ok(Self {
            content_dir: dir,
            frontmatter,
        })
    }
}

impl<F> PageWithFrontmatter<F> {
    pub fn try_render(self, renderer: &Renderer, ctx: PageContext) -> Result<RenderedPage> {
        renderer.try_render_dir(self.content_dir, ctx)
    }
}

impl<F> IntoValue for &PageWithFrontmatter<F>
where
    for<'b> &'b F: IntoValue,
{
    fn into_value(self) -> Value {
        let PageWithFrontmatter {
            content_dir: _,
            frontmatter,
        } = self;

        let mut d = Dict::new();
        d.insert("frontmatter".into(), frontmatter.into_value());
        Value::Dict(d)
    }
}

// For the moment, this function does not require access to any state.
// In case this changes in the future, make it a method of some `Parser` type.
pub fn try_parse<F>(section: Section<DiscoveredPage>) -> Result<Section<PageWithFrontmatter<F>>>
where
    F: for<'de> Deserialize<'de>,
{
    section.try_map(
        DiscoveredPage::try_parse::<F>,
        DiscoveredPage::try_parse::<F>,
    )
}
