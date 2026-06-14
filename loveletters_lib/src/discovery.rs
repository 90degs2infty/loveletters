use std::{
    collections::HashMap,
    io::ErrorKind,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::{
    error::{EntityKind, Error, Result},
    frontmatter_parsing::PageWithFrontmatter,
    page::{Index, Leaf, Mode},
    section::Section,
    slug::Slug,
};
use serde::Deserialize;
use tokio::fs::try_exists;
use tokio_stream::StreamExt;
use walkdir::{DirEntry, WalkDir};

static RESERVED_DIRS: [&str; 4] = ["_index", "posts", "static", "assets"];

pub struct Discoverer {}

impl Discoverer {
    pub async fn try_traverse(
        content_dir: &Path,
    ) -> Result<Section<DiscoveredPage<Index>, DiscoveredPage<Leaf>>> {
        // We eagerly check content_dir for existence. Note that this introduces TOCTOU bugs in case
        // content_dir is deleted afterwards (but before collecting e.g. leaf pages). However, it
        // improves error messages in the "ordinary" case, so we accept this risk.
        if !try_exists(&content_dir)
            .await
            .map_err(|e: std::io::Error| Error::FileIO {
                path: Some(content_dir.to_path_buf()),
                raw: e,
            })?
        {
            return Err(Error::NotFound {
                missing: EntityKind::ContentDirectory,
                path: Some(content_dir.to_path_buf()),
            });
        }

        Self::try_discover_section_recursive(content_dir).await
    }

    fn is_frontmatter<M: Mode>(entry: &DirEntry) -> bool {
        entry.file_type().is_file()
            && entry
                .file_name()
                .to_str()
                .is_some_and(|name| name == M::frontmatter_filename())
    }

    fn is_reserved_dir(entry: &DirEntry) -> bool {
        entry.file_type().is_file()
            && entry
                .path()
                .parent()
                .is_some_and(|p| RESERVED_DIRS.iter().any(|d| p.ends_with(d)))
    }

    fn collect_leaf_pages(dir: &Path) -> Result<HashMap<Slug, DiscoveredPage<Leaf>>> {
        WalkDir::new(dir)
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
                        let path= e.path().map(Path::to_path_buf);

                        if let Some(e) = e.io_error() && e.kind() == ErrorKind::NotFound {
                            Error::NotFound { missing: EntityKind::Other, path }
                        } else {
                        Error::FileIO {
                            path,
                            raw: e.into(),
                        }
                    }
                    }
                })?;
                // We set min_depth to 2 above, so there will always be a parent - if not, this is a logic bug
                // in our implementation. Hence, we panic instead of returning a `Result`.
                let parent_dir = entry.path().parent().unwrap_or_else(|| panic!(
                    "entry at '{}' should have a filesystem parent as filesystem is traversed with `min_depth` set to 2",
                    entry.path().display()
                ));

                println!("Collecting {}", parent_dir.display());
                let slug: Slug = parent_dir.try_into()?;
                Ok((
                    slug.clone(),
                    DiscoveredPage::<Leaf>::leaf_page(parent_dir.to_path_buf()),
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    async fn try_discover_section_recursive(
        dir: &Path,
    ) -> Result<Section<DiscoveredPage<Index>, DiscoveredPage<Leaf>>> {
        let index_page = Self::try_discover_index_page(dir.join("_index")).await?;

        let mut subsections = HashMap::new();
        let mut pages = HashMap::new();

        let mut subdirs = tokio_stream::iter(
            WalkDir::new(dir)
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_entry(|e| !Self::is_reserved_dir(e)),
        );

        // TODO process concurrently?
        while let Some(entry) = subdirs.next().await {
            let entry = entry.map_err(|e| {
                if let Some(p) = e.loop_ancestor() {
                    Error::MalformedProjectStructure {
                        path: p.to_path_buf(),
                    }
                } else {
                    let path = e.path().map(Path::to_path_buf);

                    if let Some(e) = e.io_error()
                        && e.kind() == ErrorKind::NotFound
                    {
                        Error::NotFound {
                            missing: EntityKind::Other,
                            path,
                        }
                    } else {
                        Error::FileIO {
                            path,
                            raw: e.into(),
                        }
                    }
                }
            })?;

            match Box::pin(Self::try_discover_section_recursive(entry.path())).await {
                Ok(subsection) => {
                    let slug = Slug::try_from_dir(entry.path())?;
                    subsections.insert(slug, subsection);
                }
                // If it's not a section, then maybe it's a page
                Err(Error::NotFound {
                    missing: EntityKind::Frontmatter,
                    path: _,
                }) => {
                    if let Ok(page) = Self::try_discover_leaf_page(dir.to_path_buf()).await {
                        let slug = Slug::try_from_dir(entry.path())?;
                        pages.insert(slug, page);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }

        Ok(Section::new(
            // TODO DROP
            "".to_owned().into(),
            index_page,
            pages,
            subsections,
        ))
    }

    async fn try_discover_leaf_page(dir: PathBuf) -> Result<DiscoveredPage<Leaf>> {
        let frontmatter_path = dir.join("page").with_extension("toml");

        if try_exists(&frontmatter_path)
            .await
            .map_err(|e: std::io::Error| Error::FileIO {
                path: Some(frontmatter_path.clone()),
                raw: e,
            })?
        {
            Ok(DiscoveredPage::leaf_page(dir))
        } else {
            Err(Error::NotFound {
                missing: EntityKind::Frontmatter,
                path: Some(frontmatter_path),
            })
        }
    }

    // TODO drop
    async fn try_discover_index_page(dir: PathBuf) -> Result<DiscoveredPage<Index>> {
        let frontmatter_path = dir.join("index").with_extension("toml");

        if try_exists(&frontmatter_path)
            .await
            .map_err(|e: std::io::Error| Error::FileIO {
                path: Some(frontmatter_path.clone()),
                raw: e,
            })?
        {
            Ok(DiscoveredPage::index_page(dir))
        } else {
            Err(Error::NotFound {
                missing: EntityKind::Frontmatter,
                path: Some(frontmatter_path),
            })
        }
    }
}

/// Self-contained directory representing a page.
pub struct DiscoveredPage<M> {
    content_dir: PathBuf,
    m: PhantomData<M>,
}

impl DiscoveredPage<Index> {
    /// Read an index page for the type `K`.
    pub fn index_page(dir: PathBuf) -> DiscoveredPage<Index> {
        DiscoveredPage {
            content_dir: dir,
            m: PhantomData,
        }
    }
}

impl DiscoveredPage<Leaf> {
    /// Read a leaf page from the specified directory.
    pub fn leaf_page(dir: PathBuf) -> Self {
        DiscoveredPage {
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
        PageWithFrontmatter::try_parse(self.content_dir)
    }
}
