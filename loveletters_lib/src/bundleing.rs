use std::{
    fs,
    path::{Path, PathBuf},
};

use bytes::Bytes;

use crate::{
    error::{Error, Result},
    page::{Index, Leaf},
    rendering::RenderedPage,
    section::Section,
    utils::ensure_exists,
};

pub struct InMemFile {
    content: Bytes,
}

impl InMemFile {
    pub fn new(content: Bytes) -> Self {
        Self { content }
    }

    pub fn write_to(self, out_file: &Path) -> Result<()> {
        fs::write(out_file, self.content).map_err(|e| Error::FileIO {
            path: Some(out_file.to_path_buf()),
            raw: e,
        })
    }
}

pub struct PageBundle {
    bundle_dir: PathBuf,
    index: InMemFile,
}

impl PageBundle {
    pub fn new(output_dir: PathBuf, index: InMemFile) -> Self {
        Self {
            bundle_dir: output_dir,
            index,
        }
    }

    pub fn write_to_disk(self) -> Result<()> {
        ensure_exists(&self.bundle_dir)?;
        self.index.write_to(&self.bundle_dir.join("index.html"))
    }
}

pub struct Bundler {
    root_dir: PathBuf,
    output_dir: PathBuf,
}

impl Bundler {
    pub fn new(root_dir: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            root_dir,
            output_dir,
        }
    }

    pub fn try_bundle(
        &self,
        content: Section<RenderedPage<Index>, RenderedPage<Leaf>>,
    ) -> Result<()> {
        let output_dir = self.output_dir.clone();
        let _ = content.try_walk(
            |section, rendering| {
                let output_dir = section.iter().fold(output_dir.clone(), |output_dir, slug| {
                    output_dir.join(slug.as_str())
                });
                ensure_exists(&output_dir)?;
                rendering.try_bundle(output_dir)?.write_to_disk()
            },
            |section, page, rendering| {
                let output_dir = section
                    .iter()
                    .fold(output_dir.clone(), |output_dir, slug| {
                        output_dir.join(slug.as_str())
                    })
                    .join(page.as_str());
                ensure_exists(&output_dir)?;
                rendering.try_bundle(output_dir)?.write_to_disk()
            },
        )?;

        Ok(())
    }
}
