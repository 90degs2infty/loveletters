use std::{collections::HashMap, marker::PhantomData, path::PathBuf};

use crate::{
    error::{Error, Result},
    frontmatter_parsing::PageWithFrontmatter,
    page::{Index, Leaf, Mode},
    section::Section,
    slug::Slug,
};
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};

static RESERVED_DIRS: [&'static str; 4] = ["_index", "posts", "static", "assets"];

pub struct Discoverer {}

impl Discoverer {
    pub fn try_traverse(
        content_dir: PathBuf,
    ) -> Result<Section<DiscoveredPage<Index>, DiscoveredPage<Leaf>>> {
        // TODO: implement recursively to collect sub-sections of arbitrary depth of arbitrary name
        let posts = Discoverer::collect_leaf_pages(content_dir.join("posts"))?;
        let toplevels = Discoverer::collect_leaf_pages(content_dir.clone())?;

        let posts = Section::new(
            "posts".to_owned().into(),
            DiscoveredPage::index_page(content_dir.join("posts").join("_index")),
            posts,
            HashMap::new(),
        );
        let mut sub_secs = HashMap::new();
        let _ = sub_secs.insert("posts".to_owned().into(), posts);
        let toplevel_section = Section::new(
            "".to_owned().into(),
            DiscoveredPage::index_page(content_dir.clone().join("").join("_index")),
            toplevels,
            sub_secs,
        );

        // TODO how to distinguish toplevel index and posts properly? Maybe introduce the notion of sections?
        Ok(toplevel_section)
    }

    fn is_frontmatter<M: Mode>(entry: &DirEntry) -> bool {
        entry.file_type().is_file()
            && entry
                .file_name()
                .to_str()
                .map(|name| name == M::frontmatter_filename())
                .unwrap_or(false)
    }

    fn is_reserved_dir(entry: &DirEntry) -> bool {
        entry.file_type().is_file()
            && entry
                .path()
                .parent()
                .is_some_and(|p| RESERVED_DIRS.iter().any(|d| p.ends_with(d)))
    }

    fn collect_leaf_pages(dir: PathBuf) -> Result<HashMap<Slug, DiscoveredPage<Leaf>>> {
        WalkDir::new(&dir)
            .min_depth(2)
            .max_depth(2)
            .into_iter()
            .filter_entry(|e| {
                Discoverer::is_frontmatter::<Leaf>(e) && !Discoverer::is_reserved_dir(e)
            })
            .map(|entry| {
                let entry = entry.map_err(|e| {
                    if let Some(p) = e.loop_ancestor() {
                        Error::MalformedProjectStructure {
                            path: p.to_path_buf(),
                        }
                    } else {
                        Error::FileIO {
                            path: e.path().map(|p| p.to_path_buf()),
                            raw: e.into(),
                        }
                    }
                })?;
                // We set min_depth to 2 above, so there will always be a parent - if not, this is a logic bug
                // in our implementation. Hence, we panic instead of returning a `Result`.
                let parent_dir = entry.path().parent().expect(&format!(
                    "entry at '{}' should have a filesystem parent as filesystem is traversed with `min_depth` set to 2",
                    entry.path().display()
                ));

                println!("Collecting {}", parent_dir.display());
                let slug: Slug = parent_dir.try_into()?;
                Ok((
                    slug.clone(),
                    DiscoveredPage::<Leaf>::leaf_page(slug, parent_dir.to_path_buf()),
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
    }
}

/// Self-contained directory representing a page.
pub struct DiscoveredPage<M> {
    // TODO drop
    slug: Slug,
    content_dir: PathBuf,
    m: PhantomData<M>,
}

impl DiscoveredPage<Index> {
    /// Read an index page for the type `K`.
    pub fn index_page(dir: PathBuf) -> DiscoveredPage<Index> {
        DiscoveredPage {
            slug: Slug::index(),
            content_dir: dir,
            m: PhantomData,
        }
    }
}

impl DiscoveredPage<Leaf> {
    /// Read a leaf page from the specified directory.
    pub fn leaf_page(slug: Slug, dir: PathBuf) -> Self {
        DiscoveredPage {
            slug,
            content_dir: dir,
            m: PhantomData,
        }
    }
}

impl<M: Mode> DiscoveredPage<M> {
    pub fn try_parse<F>(self) -> Result<PageWithFrontmatter<M, F>>
    where
        F: for<'de> Deserialize<'de>,
    {
        PageWithFrontmatter::try_parse(self.slug, self.content_dir)
    }
}
