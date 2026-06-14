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
    use loveletters_mock::page::Frontmatter as MockFrontmatter;
    use loveletters_test_helpers::{mismatch::Mismatch, try_match};
    use proptest::prelude::Strategy;
    use test_strategy::proptest;

    use crate::content::{IndexFrontmatter, LeafFrontmatter};

    // TODO do these unit tests serve any purpose? These are more testing serde/toml than
    // anything else...

    #[proptest]
    fn leaf_frontmatter_deserializes_valid_str(
        #[strategy(
            MockFrontmatter::builder().build()
        )]
        mock: MockFrontmatter,
    ) {
        let toml = mock.try_to_toml().expect("mock should deserialize to toml");
        let res: Result<LeafFrontmatter, _> = toml::from_str(&toml);
        try_match!(res, Ok(_))?;
    }

    #[proptest]
    fn leaf_frontmatter_rejects_missing_publication(
        #[strategy(
            MockFrontmatter::builder().without_publication().build()
        )]
        mock: MockFrontmatter,
    ) {
        let toml = mock.try_to_toml().expect("mock should deserialize to toml");
        let res: Result<LeafFrontmatter, _> = toml::from_str(&toml);

        try_match!(res, Err(_))?;
    }

    #[proptest]
    fn leaf_frontmatter_rejects_missing_title(
        #[strategy(
            MockFrontmatter::builder().without_title().build()
        )]
        mock: MockFrontmatter,
    ) {
        let toml = mock.try_to_toml().expect("mock should deserialize to toml");
        let res: Result<LeafFrontmatter, _> = toml::from_str(&toml);

        try_match!(res, Err(_))?;
    }

    #[proptest]
    fn leaf_frontmatter_rejects_invalid_publication(
        #[strategy(
            MockFrontmatter::builder().with_publication("[a-z]{4}".into_defect().boxed()).build()
        )]
        mock: MockFrontmatter,
    ) {
        let toml = mock.try_to_toml().expect("mock should deserialize to toml");
        let res: Result<LeafFrontmatter, _> = toml::from_str(&toml);

        try_match!(res, Err(_))?;
    }

    #[proptest]
    fn index_frontmatter_deserializes_valid_str(
        #[strategy(
            MockFrontmatter::builder().build()
        )]
        mock: MockFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let res: Result<IndexFrontmatter, _> = toml::from_str(&toml);
        try_match!(res, Ok(_))?;
    }

    #[proptest]
    fn index_frontmatter_rejects_missing_publication(
        #[strategy(
            MockFrontmatter::builder().without_publication().build()
        )]
        mock: MockFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let res: Result<IndexFrontmatter, _> = toml::from_str(&toml);

        try_match!(res, Err(_))?;
    }

    #[proptest]
    fn index_frontmatter_rejects_missing_title(
        #[strategy(
            MockFrontmatter::builder().without_title().build()
        )]
        mock: MockFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let res: Result<IndexFrontmatter, _> = toml::from_str(&toml);

        try_match!(res, Err(_))?;
    }

    #[proptest]
    fn index_frontmatter_rejects_invalid_publication(
        #[strategy(
            MockFrontmatter::builder().with_publication("[a-z]{4}".into_defect().boxed()).build()
        )]
        mock: MockFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let res: Result<IndexFrontmatter, _> = toml::from_str(&toml);

        try_match!(res, Err(_))?;
    }
}
