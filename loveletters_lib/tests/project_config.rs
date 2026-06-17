//! Integration tests targeting a project's toplevel configuration file.

use lattice::IntoDefect;
use loveletters_lib::error::{EntityKind, Error};
use loveletters_mock::project::{Project, ProjectStrategyBuilder};
use loveletters_test_helpers::{mismatch::Mismatch, try_match, try_render_mock};
use proptest::prelude::*;
use proptest_ext::conversion::IntoProptest;
use test_strategy::proptest;

#[proptest(async = "tokio")]
async fn project_requires_config(
    #[strategy(
        {
            ProjectStrategyBuilder::empty().without_config().build()
        }
    )]
    mock: Project,
) {
    let (_, _, res) = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::NotFound {
            missing: EntityKind::ProjectConfig,
            path: _,
        })
    )?;
}

#[proptest(async = "tokio")]
async fn project_config_requires_loveletters_filestem(
    #[strategy(
        {
            let mut project = ProjectStrategyBuilder::empty();
            project.config_filename_mut().with_stem("[a-z]{4}".boxed());
            project.build()
        }
    )]
    mock: Project,
) {
    let (_, _, res) = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::NotFound {
            missing: EntityKind::ProjectConfig,
            path: _,
        })
    )?;
}

#[proptest(async = "tokio")]
async fn project_config_requires_toml_fileext(
    #[strategy(
        {
            let mut project = ProjectStrategyBuilder::empty();
            project.config_filename_mut().with_ext("[a-z]{0,3}".boxed());
            project.build()
        }
    )]
    mock: Project,
) {
    let (_, _, res) = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::NotFound {
            missing: EntityKind::ProjectConfig,
            path: _,
        })
    )?;
}

#[proptest(async = "tokio")]
async fn project_config_requires_title(
    #[strategy(
        {
            let mut project = ProjectStrategyBuilder::empty();
            project.config_mut().expect("empty project should have a toplevel configuration").without_title();
            project.build()
        }
    )]
    mock: Project,
) {
    let (_, _, res) = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::MalformedProjectConfig {
            location: _,
            raw: _,
        })
    )?;
}

#[proptest(async = "tokio")]
async fn project_config_requires_author(
    #[strategy(
        {
            let mut project = ProjectStrategyBuilder::empty();
            project.config_mut().expect("empty project should have a toplevel configuration").without_author();
            project.build()
        }
    )]
    mock: Project,
) {
    let (_, _, res) = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::MalformedProjectConfig {
            location: _,
            raw: _,
        })
    )?;
}

#[proptest(async = "tokio")]
async fn project_config_requires_root(
    #[strategy(
        {
            let mut project = ProjectStrategyBuilder::empty();
            project.config_mut().expect("empty project should have a toplevel configuration").without_root();
            project.build()
        }
    )]
    mock: Project,
) {
    let (_, _, res) = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::MalformedProjectConfig {
            location: _,
            raw: _,
        })
    )?;
}

#[proptest(async = "tokio")]
async fn project_config_requires_valid_root(
    #[strategy(
        {
            let mut project = ProjectStrategyBuilder::empty();
            project.config_mut().expect("empty project should have a toplevel configuration").with_root("[a-z]*".into_defect().boxed());
            project.build()
        }
    )]
    mock: Project,
) {
    let (_, _, res) = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::MalformedProjectConfig {
            location: _,
            raw: _,
        })
    )?;
}

#[proptest(async = "tokio")]
async fn project_config_denies_excess_project_config_key(
    #[strategy(
        {
            let mut project = ProjectStrategyBuilder::empty();
            project.config_mut().expect("empty project should have a toplevel configuration").with_excess("[a-z]{0,4}".prop_map(Some).boxed());
            project.build()
        }
    )]
    mock: Project,
) {
    let (_, _, res) = try_render_mock(&mock).await.into_proptest()?;

    try_match!(
        res,
        Err(Error::MalformedProjectConfig {
            location: _,
            raw: _,
        })
    )?;
}
