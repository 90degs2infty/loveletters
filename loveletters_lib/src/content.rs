use serde::{Deserialize, Serialize};
use time::{Date, PrimitiveDateTime, Time};
use typst::foundations::{Datetime, Dict, IntoValue, Value};

// TODO: dedicated module?
// TODO: should (maybe) be empty instead - how to tell serde?
#[derive(Debug, Deserialize, Serialize)]
pub struct IndexFrontmatter {
    title: String,
    publication: Date,
    // TODO expiry: OffsetDateTime,
}

impl IndexFrontmatter {
    pub fn to_typst(&self) -> Value {
        let Self { title, publication } = self;

        let mut d = Dict::new();
        d.insert("title".into(), Value::Str(title.as_str().into()));
        d.insert(
            "publication".into(),
            Value::Datetime(Datetime::Datetime(PrimitiveDateTime::new(
                *publication,
                Time::MIDNIGHT,
            ))),
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
    publication: Date,
    // TODO expiry: OffsetDateTime,
}

impl LeafFrontmatter {
    pub fn to_typst(&self) -> Value {
        let Self { title, publication } = self;

        let mut d = Dict::new();
        d.insert("title".into(), Value::Str(title.as_str().into()));
        d.insert(
            "publication".into(),
            Value::Datetime(Datetime::Datetime(PrimitiveDateTime::new(
                *publication,
                Time::MIDNIGHT,
            ))),
        );
        Value::Dict(d)
    }
}

impl IntoValue for &LeafFrontmatter {
    fn into_value(self) -> Value {
        self.to_typst()
    }
}

#[cfg(test)]
mod tests {
    use lattice::IntoDefect;
    use loveletters_mock::leaf::Frontmatter as MockLeafFrontmatter;
    use proptest::prelude::Strategy;
    use test_strategy::proptest;

    use crate::content::LeafFrontmatter;

    #[derive(Debug)]
    struct UnexpectedResult<T, E> {
        inner: Result<T, E>,
    }

    #[proptest]
    fn leaf_frontmatter_deserializes_valid_str(
        #[strategy(
            MockLeafFrontmatter::builder().build()
        )]
        mock: MockLeafFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let _: LeafFrontmatter = toml::from_str(&toml)?;
    }

    #[proptest]
    fn leaf_frontmatter_rejects_missing_publication(
        #[strategy(
            MockLeafFrontmatter::builder().without_publication().build()
        )]
        mock: MockLeafFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let _: LeafFrontmatter = toml::from_str(&toml)?;
        // TODO assert the right thing
    }

    #[proptest]
    fn leaf_frontmatter_rejects_missing_title(
        #[strategy(
            MockLeafFrontmatter::builder().without_title().build()
        )]
        mock: MockLeafFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let _: LeafFrontmatter = toml::from_str(&toml)?;
        // TODO assert the right thing
    }

    #[proptest]
    fn leaf_frontmatter_rejects_invalid_publication(
        #[strategy(
            MockLeafFrontmatter::builder().with_publication("[a-z]{4}".into_defect().boxed()).build()
        )]
        mock: MockLeafFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let _: LeafFrontmatter = toml::from_str(&toml)?;
        // TODO assert the right thing
    }
}
