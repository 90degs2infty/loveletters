use crate::error::{Error, Result};
use std::{ffi::OsStr, path::Path};

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Slug(String);

impl Slug {
    pub fn try_from_dir<T: AsRef<Path>>(path: T) -> Result<Self> {
        let slug = path
            .as_ref()
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| Error::InvalidSlug {
                path: path.as_ref().to_path_buf(),
            })?
            .to_owned();
        Ok(Self(slug))
    }

    pub fn index() -> Self {
        Self("_index".to_owned())
    }

    pub fn from_string(slug: String) -> Self {
        Self(slug)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&Path> for Slug {
    type Error = Error;
    fn try_from(value: &Path) -> Result<Self> {
        Slug::try_from_dir(value)
    }
}

impl From<String> for Slug {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
