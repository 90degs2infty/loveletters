//! loveletters' heavy lifting.

mod bundleing;
mod config;
mod content;
mod discovery;
pub mod error;
mod frontmatter_parsing;
mod page;
mod rendering;
pub(crate) mod seal;
mod section;
mod slug;
mod utils;

use crate::{
    bundleing::Bundler,
    config::Config,
    discovery::Discoverer,
    error::{EntityKind, Error, Result},
    frontmatter_parsing::try_parse as try_parse_frontmatter,
    rendering::{Renderer, context::ProjectContext},
    utils::ensure_exists,
};
use std::path::Path;

/// Render `loveletters` project at `input_dir` and write rendered output to `output_dir`.
///
/// # Errors
///
/// Returns an [`Error`] when encountering failures states as defined by [`Error`].
pub fn render_dir(input_dir: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> Result<()> {
    let input_dir = input_dir.as_ref().canonicalize().map_err(|e| {
        Error::from_io_error(
            e,
            Some(input_dir.as_ref().into()),
            EntityKind::InputDirectory,
        )
    })?;

    let output_dir = output_dir.as_ref().canonicalize().map_err(|e| {
        Error::from_io_error(
            e,
            Some(output_dir.as_ref().into()),
            EntityKind::OutputDirectory,
        )
    })?;
    ensure_exists(&output_dir)?;

    let content_dir = input_dir.join("content");

    let config = Config::try_read_from_disk(&input_dir.join("loveletters.toml"))?;

    let bundler = Bundler::new(output_dir.clone());

    let discovered_content = Discoverer::try_traverse(&content_dir)?;
    let frontmatter = try_parse_frontmatter(discovered_content)?;
    let global_ctx = ProjectContext::new(&frontmatter, config);
    let renderer = Renderer::new(global_ctx, input_dir.join("packages"));
    let rendering = renderer.try_render(frontmatter)?;
    bundler.try_bundle(rendering)
}
