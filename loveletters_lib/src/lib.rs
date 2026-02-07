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
use std::{io::ErrorKind, path::PathBuf};

/// Render `loveletters` project at `input_dir` and write rendered output to `output_dir`.
///
/// # Errors
///
/// Returns an [`Error`] when encountering failures states as defined by [`Error`].
pub fn render_dir(input_dir: PathBuf, output_dir: PathBuf) -> Result<()> {
    let input_dir = &input_dir.canonicalize().map_err(|e| match e.kind() {
        ErrorKind::NotFound => Error::NotFound {
            missing: EntityKind::InputDirectory,
            path: input_dir,
        },
        _ => Error::FileIO {
            path: Some(input_dir),
            raw: e,
        },
    })?;

    let output_dir = &output_dir.canonicalize().map_err(|e| match e.kind() {
        ErrorKind::NotFound => Error::NotFound {
            missing: EntityKind::OutputDirectory,
            path: output_dir,
        },
        _ => Error::FileIO {
            path: Some(output_dir),
            raw: e,
        },
    })?;
    ensure_exists(output_dir)?;

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
