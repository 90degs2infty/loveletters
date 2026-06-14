//! Integration test targeting overall content structure.

use anyhow::Context;
use loveletters_lib::error::{EntityKind, Error};
use loveletters_mock::project::{Project, ProjectStrategyBuilder};
use loveletters_test_helpers::{mismatch::Mismatch, try_match, try_render_mock};
use proptest_ext::conversion::IntoProptest;
use test_strategy::proptest;
#[proptest(async = "tokio")]
async fn project_requires_content(
    #[strategy(
        {
            ProjectStrategyBuilder::empty().without_content().build()
        }
    )]
    mock: Project,
) {
    let res = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::NotFound {
            missing: EntityKind::ContentDirectory,
            path: _,
        })
    )?;
}

#[ignore = "requires generalization of content discovery"]
#[proptest(async = "tokio")]
async fn project_requires_nonempty_content_dir(
    #[strategy(
        {
            ProjectStrategyBuilder::empty().without_content().enforce_content_dir().build()
        }
    )]
    mock: Project,
) {
    let res = try_render_mock(&mock)
        .await
        .with_context(|| "while rendering the mock project")
        .into_proptest()?;

    try_match!(
        res,
        Err(Error::NotFound {
            missing: EntityKind::ToplevelSectionIndex,
            path: _,
        })
    )?;
}
