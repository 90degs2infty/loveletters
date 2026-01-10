use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use typst::foundations::{Dict, IntoValue, Value};
use url::{Position, Url};

use crate::error::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    title: String,
    author: String,
    root: Url,
}

impl Config {
    pub fn try_read_from_disk(path: &Path) -> Result<Self> {
        let config: String = fs::read_to_string(path).expect(&format!(
            "Failed to open config file at '{}'",
            path.display()
        ));
        let config = toml::from_str(&config).expect(&format!(
            "Failed to parse config from file at '{}'",
            path.display()
        ));
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
