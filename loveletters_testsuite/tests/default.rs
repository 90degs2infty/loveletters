//! Default set of test cases.

use loveletters_lib::{
    error::{Error, Result},
    render_dir,
};
use loveletters_testsuite::mock::{LeafFrontmatter, LeafPage, Project, Section, TypstFile};
use proptest::prelude::*;
use std::{fs, mem, path::PathBuf};
use tempfile::{Builder, TempDir};

fn replace_random_leaf(
    section: impl Strategy<Value = Section>,
    leaf: impl Strategy<Value = LeafPage>,
) -> impl Strategy<Value = Section> {
    (
        section
            // Note on filter: it is somewhat unlikely to encounter a "recursively" empty section,
            // i.e. a section where the section itself and all subsections do not feature any leaf
            // pages. Thus, we stick to the filter for the moment. Another approach is to just
            // append a new (invalid) page at some arbitrary location in case one encounters an
            // empty section.
            .prop_filter("(recursively) empty section, no leaf to replace", |sec| {
                sec.num_leafs() != 0
            })
            .prop_flat_map(|sec| {
                let num_leafs = sec.num_leafs();
                (Just(sec), 0..num_leafs)
            }),
        leaf,
    )
        .prop_flat_map(|((mut sec, idx), leaf)| {
            let _ = mem::replace(sec.leaf_at_mut(idx).expect("msg"), leaf);
            Just(sec)
        })
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

proptest! {
    #[test]
    fn project_requires_configuration(project in Project::missing_config()) {
        let (_input, _output, res) = render_project(&project);

        let matches = matches!(
            res,
            Err(Error::NotFound { missing: loveletters_lib::error::EntityKind::ProjectConfig, path: _ })
        );

        prop_assert!(matches)
    }

    #[test]
    fn project_requires_content(project in Project::missing_content()) {
        let (_input, _output, res) = render_project(&project);

        let matches = matches!(
            res,
            Err(Error::NotFound { missing: loveletters_lib::error::EntityKind::ContentDirectory, path: _ })
        );

        prop_assert!(matches)
    }

}
