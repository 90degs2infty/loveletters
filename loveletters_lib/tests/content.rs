//! Integration test targeting overall content structure.

use lattice::IntoDefect;
use loveletters_lib::error::{EntityKind, Error};
use loveletters_mock::project::{Project, ProjectStrategyBuilder};
use loveletters_test_helpers::{into_unexpected, try_render_mock, unexpected::Unexpected};
use proptest::prelude::*;
use proptest_ext::conversion::IntoProptest;
use std::result::Result;
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

    into_unexpected!(
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
    let res = try_render_mock(&mock).await.into_proptest()?;

    into_unexpected!(
        res,
        Err(Error::NotFound {
            missing: EntityKind::ToplevelSectionIndex,
            path: _,
        })
    )?;
}
