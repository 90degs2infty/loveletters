use loveletters_testsuite::mock::{LeafFrontmatter, LeafPage, Section, TypstFile};
use proptest::prelude::*;
use std::{fs, mem, path::PathBuf};

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

proptest! {
    #[test]
    fn write_section(section in replace_random_leaf(
        any::<Section>(),
        LeafPage::general(
            LeafFrontmatter::missing_title().prop_map(Option::Some),
            any::<TypstFile>().prop_map(Option::Some)
        )
    ), dir_name in "[a-z]{6}") {
        let tmp_dir = PathBuf::from("out").join(dir_name);
        fs::create_dir(&tmp_dir).unwrap();
        // let tmp_dir = Builder::new().prefix("loveletters").tempdir().unwrap();
        section.write_to_dir(tmp_dir.as_ref());
    }
}
