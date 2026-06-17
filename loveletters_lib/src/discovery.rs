use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use crate::{
    error::{EntityKind, Error, Result},
    frontmatter_parsing::PageWithFrontmatter,
    section::Section,
    slug::Slug,
};
use serde::Deserialize;
use tokio::fs::try_exists;
use tokio_stream::StreamExt;
use walkdir::{DirEntry, WalkDir};

const RESERVED_DIRS: [&str; 4] = ["_index", "posts", "static", "assets"];

pub struct Discoverer {}

impl Discoverer {
    pub async fn try_traverse(content_dir: &Path) -> Result<Section<DiscoveredPage>> {
        // We eagerly check content_dir for existence. Note that this introduces TOCTOU bugs in case
        // content_dir is deleted afterwards (but before collecting e.g. leaf pages). However, it
        // improves error messages in the "ordinary" case, so we accept this risk.
        if !try_exists(&content_dir)
            .await
            .map_err(|e: io::Error| Error::FileIO {
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

    fn is_reserved_dir(entry: &DirEntry) -> bool {
        entry.file_type().is_dir() && RESERVED_DIRS.iter().any(|d| entry.path().ends_with(d))
    }

    async fn try_discover_section_recursive(dir: &Path) -> Result<Section<DiscoveredPage>> {
        let index_page = Self::try_discover_page(dir.join("_index")).await?;

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
                    if let Ok(page) = Self::try_discover_page(entry.path().to_path_buf()).await {
                        let slug = Slug::try_from_dir(entry.path())?;
                        pages.insert(slug, page);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(Section::new(index_page, pages, subsections))
    }

    async fn try_discover_page(dir: PathBuf) -> Result<DiscoveredPage> {
        let frontmatter_path = dir.join("page").with_extension("toml");

        if try_exists(&frontmatter_path)
            .await
            .map_err(|e: io::Error| Error::FileIO {
                path: Some(frontmatter_path.clone()),
                raw: e,
            })?
        {
            Ok(DiscoveredPage::new(dir))
        } else {
            Err(Error::NotFound {
                missing: EntityKind::Frontmatter,
                path: Some(frontmatter_path),
            })
        }
    }
}

/// Self-contained directory representing a page.
pub struct DiscoveredPage {
    content_dir: PathBuf,
}

impl DiscoveredPage {
    /// Read a content page from `dir`.
    pub fn new(dir: PathBuf) -> Self {
        Self { content_dir: dir }
    }
}

impl DiscoveredPage {
    pub fn try_parse<F>(self) -> Result<PageWithFrontmatter<F>>
    where
        F: for<'de> Deserialize<'de>,
    {
        PageWithFrontmatter::try_parse(self.content_dir)
    }
}
