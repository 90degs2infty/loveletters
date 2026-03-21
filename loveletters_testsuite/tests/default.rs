//! Default set of test cases.

use loveletters_lib::{
    error::{EntityKind, Error, Result},
    render_dir,
};
use loveletters_testsuite::mock::{
    LeafFrontmatter, LeafPage, Project, ProjectConfig, Section, Slug, TypstFile,
};
use proptest::prelude::*;
use std::mem;
use tempfile::{Builder, TempDir};
use test_strategy::proptest;

macro_rules! prop_assert_matches {
    ( $e:expr , $pat:pat ) => {{
        let matches = matches!($e, $pat);

        prop_assert!(
            matches,
            "{:?} does not match pattern {}",
            $e,
            stringify!($pat)
        )
    }};
}

#[derive(Debug)]
enum Location {
    ReplaceLeaf(usize),
    AppendToSection(usize, Slug),
}

fn attach_mutation_location(sec: Section) -> impl Strategy<Value = (Section, Location)> {
    if sec.num_leafs() > 0 {
        let num_leafs = sec.num_leafs();
        (Just(sec), (0..num_leafs).prop_map(Location::ReplaceLeaf)).boxed()
    } else {
        let num_sections = sec.num_sections();
        (
            Just(sec),
            (0..num_sections, Slug::valid())
                .prop_map(|(idx, slug)| Location::AppendToSection(idx, slug)),
        )
            .boxed()
    }
}

fn replace_random_leaf(
    section: impl Strategy<Value = Section>,
    leaf: impl Strategy<Value = LeafPage>,
) -> impl Strategy<Value = Section> {
    (section.prop_flat_map(attach_mutation_location), leaf).prop_flat_map(
        |((mut sec, loc), leaf)| match loc {
            Location::ReplaceLeaf(idx) => {
                let _ = mem::replace(
                    sec.leaf_at_mut(idx)
                        .expect("index should point to valid leaf page"),
                    leaf,
                );
                Just(sec)
            }
            Location::AppendToSection(idx, slug) => {
                let previous = sec
                    .section_at_mut(idx)
                    .expect("index should point to valid (sub-)section")
                    .insert_leaf(slug, leaf);

                // As there is no expect_none (yet), we panic by hand...
                if let Some(_) = previous {
                    panic!("key should not point to pre-existing leaf page");
                }

                Just(sec)
            }
        },
    )
}

fn setup_testcase(project: &Project) -> (TempDir, TempDir) {
    let input_dir = Builder::new().prefix("loveletters").tempdir().unwrap();
    let output_dir = Builder::new().prefix("loveletters").tempdir().unwrap();
    project.write_to_dir(input_dir.as_ref());

    (input_dir, output_dir)
}

fn render_project(project: &Project) -> (TempDir, TempDir, Result<()>) {
    let (input, output) = setup_testcase(project);
    let res = render_dir(&input, &output);
    (input, output, res)
}
#[proptest]
fn project_requires_configuration(#[strategy(Project::missing_config())] project: Project) {
    let (_input, _output, res) = render_project(&project);

    prop_assert_matches!(
        res,
        Err(Error::NotFound {
            missing: EntityKind::ProjectConfig,
            path: _
        })
    )
}

#[proptest]
fn project_requires_content(#[strategy(Project::missing_content())] project: Project) {
    let (_input, _output, res) = render_project(&project);

    prop_assert_matches!(
        res,
        Err(Error::NotFound {
            missing: EntityKind::ContentDirectory,
            path: _
        })
    )
}

// TODO: does it make more sense to have dedicated test cases for distinct cases of broken project configs?
// Given the number of iterations during test case execution: probably not
#[proptest]
fn project_requires_valid_frontmatter(
    #[strategy(
        Project::general(
            Section::valid().prop_map(Option::Some),
            ProjectConfig::invalid().prop_map(Option::Some)
        )
    )]
    project: Project,
) {
    let (_input, _output, res) = render_project(&project);

    prop_assert_matches!(
        res,
        Err(Error::MalformedProjectConfig {
            location: _,
            raw: _
        })
    )
}

#[proptest]
fn leaf_page_requires_valid_frontmatter(
    #[strategy(
        Project::general(
            replace_random_leaf(
                Section::toplevel_and_posts(),
                LeafPage::general(
                    LeafFrontmatter::invalid().prop_map(Option::Some),
                    TypstFile::valid().prop_map(Option::Some)
                )
            ).prop_map(Option::Some),
            ProjectConfig::valid().prop_map(Option::Some)
        )
    )]
    project: Project,
) {
    let (_input, _output, res) = render_project(&project);

    prop_assert_matches!(
        res,
        Err(Error::MalformedFrontmatter {
            location: _,
            raw: _
        })
    )
}

// Further test cases
//
// - page inside section directory
// - broken project config
//   - subcases individually or the general invalid strategy?
// - section without index page (should not error, but not create any output for said section)
// - section with broken index front matter
// - leaf page without frontmatter file (should not error, but not create any output for said page)
