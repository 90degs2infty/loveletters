use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, PrimitiveDateTime};
use typst::foundations::{Datetime, Dict, IntoValue, Value};

// TODO: dedicated module?
// TODO: should (maybe) be empty instead - how to tell serde?
#[derive(Debug, Deserialize, Serialize)]
pub struct IndexFrontmatter {
    title: String,
    #[serde(with = "time::serde::iso8601")]
    publication: OffsetDateTime,
    // TODO expiry: OffsetDateTime,
}

impl IndexFrontmatter {
    pub fn to_typst(&self) -> Value {
        let Self { title, publication } = self;

        let mut d = Dict::new();
        d.insert("title".into(), Value::Str(title.as_str().into()));
        d.insert(
            "publication".into(),
            Value::Datetime(Datetime::Datetime(
                // TODO is this the intended way to (serde) deserialize a date and get a datetime from it?
                PrimitiveDateTime::new(publication.date(), publication.time()),
            )),
        );
        Value::Dict(d)
    }
}

impl IntoValue for &IndexFrontmatter {
    fn into_value(self) -> Value {
        self.to_typst()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeafFrontmatter {
    title: String,
    #[serde(with = "time::serde::iso8601")]
    publication: OffsetDateTime,
    // TODO expiry: OffsetDateTime,
}

impl LeafFrontmatter {
    pub fn to_typst(&self) -> Value {
        let Self { title, publication } = self;

        let mut d = Dict::new();
        d.insert("title".into(), Value::Str(title.as_str().into()));
        d.insert(
            "publication".into(),
            Value::Datetime(Datetime::Datetime(
                // TODO is this the intended way to (serde) deserialize a date and get a datetime from it?
                PrimitiveDateTime::new(publication.date(), publication.time()),
            )),
        );
        Value::Dict(d)
    }
}

impl IntoValue for &LeafFrontmatter {
    fn into_value(self) -> Value {
        self.to_typst()
    }
}
