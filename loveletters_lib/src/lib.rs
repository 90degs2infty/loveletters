//! loveletters' heavy lifting.

mod bundleing;
mod content;
mod discovery;
mod driver_typst;
pub mod error;
mod frontmatter_parsing;
mod page;
mod rendering;
pub(crate) mod seal;
mod section;
mod slug;
mod utils;

use crate::{
    bundleing::Bundler, discovery::Discoverer, error::Result, frontmatter_parsing::Parser,
    rendering::Renderer, utils::ensure_exists,
};
use std::path::PathBuf;

pub fn render_dir(input_dir: PathBuf, output_dir: PathBuf) -> Result<()> {
    let input_dir = input_dir
        .canonicalize()
        .expect("Could not resolve specified input directory");

    ensure_exists(&output_dir)?;
    let output_dir = output_dir
        .canonicalize()
        .expect("Could not resolve specified output directory");

    let content_dir = input_dir.join("content");

    let parser = Parser::new();
    let renderer = Renderer::new();
    let bundler = Bundler::new(input_dir.clone(), output_dir);

    let discovered_content = Discoverer::try_traverse(content_dir)?;
    let frontmatter = parser.try_parse(discovered_content)?;
    let rendering = renderer.try_render(frontmatter)?;
    bundler.try_bundle(rendering)
}
