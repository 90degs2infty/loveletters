use serde::{Deserialize, Serialize};
use typst::foundations::{Dict, IntoValue, Value};

// TODO: dedicated module?
// TODO: should (maybe) be empty instead - how to tell serde?
#[derive(Debug, Deserialize, Serialize)]
pub struct IndexFrontmatter {
    title: String,
}

impl IndexFrontmatter {
    pub fn to_typst(&self) -> Value {
        let Self { title } = self;

        let mut d = Dict::new();
        d.insert(
            "title".into(),
            typst::foundations::Value::Str(title.as_str().into()),
        );
        Value::Dict(d)
    }
}

impl IntoValue for IndexFrontmatter {
    fn into_value(self) -> Value {
        self.to_typst()
    }
}

// TODO: should rather be a custom trait ToValue?
impl<'a> IntoValue for &'a IndexFrontmatter {
    fn into_value(self) -> Value {
        self.to_typst()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeafFrontmatter {
    title: String,
}

impl LeafFrontmatter {
    pub fn to_typst(&self) -> Value {
        let Self { title } = self;

        let mut d = Dict::new();
        d.insert(
            "title".into(),
            typst::foundations::Value::Str(title.as_str().into()),
        );
        Value::Dict(d)
    }
}

impl IntoValue for LeafFrontmatter {
    fn into_value(self) -> Value {
        self.to_typst()
    }
}

// TODO: should rather be a custom trait ToValue?
impl<'a> IntoValue for &'a LeafFrontmatter {
    fn into_value(self) -> Value {
        self.to_typst()
    }
}
