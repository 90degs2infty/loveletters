use crate::error::{EntityKind, Error, Result};
use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind, path::Path};
use typst::foundations::{Dict, IntoValue, Value};
use url::{Position, Url};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    title: String,
    author: String,
    root: Url,
}

impl Config {
    pub fn try_read_from_disk(path: &Path) -> Result<Self> {
        let config: String = fs::read_to_string(path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => Error::NotFound {
                missing: EntityKind::ProjectConfig,
                path: path.to_path_buf(),
            },
            _ => Error::FileIO {
                path: Some(path.to_path_buf()),
                raw: e,
            },
        })?;
        let config = toml::from_str(&config).map_err(|e| Error::MalformedProjectConfig {
            location: path.to_path_buf(),
            // No need to attach additional context, as the context is represented by
            // the containing error
            raw: e.into(),
        })?;
        Ok(config)
    }

    pub fn to_typst(&self) -> Dict {
        let Self {
            title,
            author,
            root,
        } = self;

        let mut root_dict = Dict::new();

        let server: &str = &root[..Position::BeforePath];
        let path: &str = &root[Position::BeforePath..Position::AfterPath];
        root_dict.insert("server".into(), Value::Str(server.into()));
        root_dict.insert("path".into(), Value::Str(path.into()));

        let mut d = Dict::new();
        d.insert("root".into(), root_dict.into_value());
        d.insert("author".into(), author.as_str().into_value());
        d.insert("title".into(), title.as_str().into_value());

        d
    }
}
