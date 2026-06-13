//! Helpers for implementing tests targeting `loveletters`.

pub mod mismatch;

use anyhow::{Context, Result};
use loveletters_lib::{error::Error, render_dir};
use loveletters_mock::project::Project;
use std::result::Result as StdResult;
use tempfile::TempDir;

/// Try to render the specified `mock` project.
///
/// Writes the specified `mock` to a temporary directory and renders the resulting content to a
/// temporary output directory.
/// Returns the result of calling [`render_dir`] (wrapped as `Ok(res)`) in case content preparation
/// is successful.
///
/// # Errors
///
/// Returns an error if for some reason writing `mock` fails.
///
/// Note that potential errors returned by [`render_dir`] are _not_ returned as error!
/// I.e. the result returned by [`render_dir`] is passed on unchanged wrapped in `Ok(res)`.
pub async fn try_render_mock(mock: &Project) -> Result<StdResult<(), Error>> {
    let input_dir = TempDir::with_prefix("loveletters-")
        .with_context(|| "while creating a temporary input directory")?;

    let output_dir = TempDir::with_prefix("loveletters-")
        .with_context(|| "while creating a temporary output directory")?;

    mock.try_write_to_dir(input_dir.as_ref())
        .await
        .with_context(|| {
            format!(
                "while writing project to temporary directory {}",
                input_dir.as_ref().display()
            )
        })?;

    let res = render_dir(input_dir.path(), output_dir.path());

    Ok(res)
}
