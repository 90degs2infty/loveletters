//! Integration test targeting overall content structure.

use anyhow::Context;
use loveletters_lib::error::{EntityKind, Error};
use loveletters_mock::{
    page::Page,
    project::{Project, ProjectStrategyBuilder},
    typst::snippet::StrategyKind as SnippetKind,
};
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
            missing: EntityKind::Frontmatter,
            path: _,
        })
    )?;
}

#[proptest(async = "tokio")]
async fn render_dir_accepts_arbitrary_project_structure(
    #[strategy(
        {
            let mut page = Page::builder();
            page.content_mut().expect("page should have content").push_snippet(SnippetKind::Lorem.into_strategy());

            let mut project = ProjectStrategyBuilder::empty();
            project.content_mut().expect("project should have content").draw_pages(page, 0..5).recurse(3, 4, 2);
            project.build()
        }
    )]
    mock: Project,
) {
    let res = try_render_mock(&mock)
        .await
        .with_context(|| "while rendering the mock project")
        .into_proptest()?;

    try_match!(res, Ok(()))?;
}
