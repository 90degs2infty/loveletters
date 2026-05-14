//! Default set of test cases.

use anyhow::Error as AError;
use loveletters_lib::{
    error::{EntityKind, Error, Result},
    render_dir,
};
use loveletters_testsuite::mock_outdated::{
    LeafFrontmatter, LeafPage, Project, ProjectConfig, Section, Slug, TypstFile,
};
use proptest::{prelude::*, test_runner::TestCaseResult};
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

fn anyhow_into_proptest(e: AError) -> TestCaseError {
    TestCaseError::fail(format!("{e:#}"))
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

// TODO: as of now, the rendering of typst content is rather slow. Improve processing speed and
// increase the number of cases to the default again.
#[proptest(ProptestConfig { cases : 10, ..ProptestConfig::default() })]
fn leaf_page_requires_valid_typst_source(
    #[strategy(
        Project::general(
            replace_random_leaf(
                Section::toplevel_and_posts(),
                LeafPage::general(
                    LeafFrontmatter::valid().prop_map(Option::Some),
                    TypstFile::invalid().prop_map(Option::Some)
                )
            ).prop_map(Option::Some),
            ProjectConfig::valid().prop_map(Option::Some)
        )
    )]
    project: Project,
) {
    let (_input, _output, res) = render_project(&project);

    prop_assert_matches!(res, Err(Error::Compilation { page: _, raw: _ }))
}

// TODO: as of now, the rendering of typst content is rather slow. Improve processing speed and
// increase the number of cases to the default again.
#[proptest(ProptestConfig { cases : 10, ..ProptestConfig::default() })]
fn leaf_page_requires_typst_source_file(
    #[strategy(
        Project::general(
            replace_random_leaf(
                Section::toplevel_and_posts(),
                LeafPage::general(
                    LeafFrontmatter::valid().prop_map(Option::Some),
                    Just(None),
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
        Err(Error::NotFound {
            missing: EntityKind::TypstRoot,
            path: _,
        })
    )
}

// TODO: as of now, the rendering of typst content is rather slow. Improve processing speed and
// increase the number of cases to the default again.
#[proptest(ProptestConfig { cases : 10, ..ProptestConfig::default() })]
fn leaf_page_lacking_frontmatter_is_ignored(
    #[strategy(
        Project::general(
            replace_random_leaf(
                Section::toplevel_and_posts(),
                LeafPage::general(
                    Just(None),
                    TypstFile::valid().prop_map(Option::Some),
                )
            ).prop_map(Option::Some),
            ProjectConfig::valid().prop_map(Option::Some)
        )
    )]
    project: Project,
) {
    let (_input, out_dir, res) = render_project(&project);

    prop_assert_matches!(res, Ok(()));

    project
        .verify_output_bundle_present(out_dir.as_ref())
        .map_err(anyhow_into_proptest)?
}

#[ignore = "with the current processing, the toplevel `index.html` cannot be attributed exactly one of the toplevel section or the sole leaf page"]
// TODO: as of now, the rendering of typst content is rather slow. Improve processing speed and
// increase the number of cases to the default again.
#[proptest(ProptestConfig { cases: 10, ..ProptestConfig::default()})]
fn leaf_page_in_section_directory_is_ignored(
    #[strategy(Project::general(
        Section::toplevel_and_posts().prop_map(Option::Some),
        ProjectConfig::valid().prop_map(Option::Some)
    ))]
    project: Project,
    #[strategy(LeafPage::valid())] leaf: LeafPage,
) -> TestCaseResult {
    let (in_dir, out_dir) = setup_testcase(&project);

    // TODO to improve coverage, consider writing the leaf page to some arbitrary section somewhere
    // within the content tree (not always the root section).
    leaf.write_to_dir(&in_dir.as_ref().join("content"));

    let res = render_dir(&in_dir, &out_dir);
    prop_assert_matches!(res, Ok(()));
    let () = project
        .verify_output_bundle_present(out_dir.as_ref())
        .map_err(anyhow_into_proptest)?;

    // TODO: everything except the index page has to be missing. However, the index page exists
    // because of the toplevel section's index page being placed at the root output directory.
    //
    // Improve testability by separating content discovery (and checking that the leaf page is
    // missing from the discovered content) from content rendering.
    let () = leaf
        .verify_output_bundle_missing(out_dir.as_ref())
        .map_err(|e| e.context("while checking sole leaf page"))
        .map_err(anyhow_into_proptest)?;
    Ok(())
}

#[proptest]
fn section_without_index_frontmatter_is_ignored(
    #[strategy(Project::general(
        Section::toplevel_and_posts().prop_map(Option::Some),
        ProjectConfig::valid().prop_map(Option::Some)
    ).prop_flat_map(|p| {
        let num_sections = p.content().expect("valid project should have content").num_sections();
        // TODO empty case
        (Just(p), 0..num_sections).prop_map(|(mut p, idx)| {
            let _ = p.content_mut().expect("valid project should have content").section_at_mut(idx).expect("index should point to valid subsection").index_mut().expect("valid section should have index page").without_frontmatter();
            p
        })
    }))]
    project: Project,
) {
    // only index frontmatter missing (i.e. there is Some(IndexPage))
    let (_input, out_dir, res) = render_project(&project);

    prop_assert_matches!(res, Ok(()));

    project
        .verify_output_bundle_present(out_dir.as_ref())
        .map_err(anyhow_into_proptest)?
}

#[proptest]
fn section_without_index_page_is_ignored() {
    // entire page missing (i.e. None)
    prop_assert!(false)
}

#[proptest]
fn section_index_page_requires_valid_frontmatter() {
    prop_assert!(false)
}

#[proptest]
fn section_index_page_requires_valid_typst_source() {
    prop_assert!(false)
}

#[proptest]
fn section_index_page_requires_typst_source_file() {
    prop_assert!(false)
}

#[proptest]
fn valid_input_maps_to_valid_output() {
    prop_assert!(false)
}

// TODO: once content discovery has been generalized to arbitrary content structures,
// rework your testcases: testcases targeting content discovery should remain "complex" in
// structure (i.e. use a recursive strategy), testcases targeting other functionality should be
// simplified to e.g. single-section single-leaf content trees.

// TODO (separate issue/PR): split processing into content discovery and rendering. Then implement
// the testcases ignored above. Also, identify the testcases targeting content discovery and skip
// rendering for those.
