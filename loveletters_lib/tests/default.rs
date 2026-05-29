use tempfile::TempDir;
use test_strategy::proptest;

use loveletters_mock::{
    page::leaf::PageStrategyBuilder,
    section::{Section, StrategyBuilder as SectionStrategyBuilder},
};

#[proptest(async = "tokio")]
async fn serialize_section(
    #[strategy(
        {
            let mut page = PageStrategyBuilder::valid();
            let mut section = SectionStrategyBuilder::empty();
            section.draw_pages(page, 0..5);
            // section.recurse(5, 32, 3);
            section.build()
        }
    )]
    mock: Section,
) {
    let temp_dir = TempDir::with_prefix("loveletters-").expect("temp dir creation should succeed");
    mock.try_write_to_dir(temp_dir.as_ref())
        .await
        .expect("writing should suceed");
    std::mem::forget(temp_dir);
}
