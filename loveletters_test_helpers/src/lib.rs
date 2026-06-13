pub mod mismatch;

use anyhow::{Context, Result};
use loveletters_lib::{error::Error, render_dir};
use loveletters_mock::project::Project;
use std::result::Result as StdResult;
use tempfile::TempDir;

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
