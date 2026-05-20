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
    use loveletters_mock::page::index::Frontmatter as MockIndexFrontmatter;
    use loveletters_mock::page::leaf::Frontmatter as MockLeafFrontmatter;
    use proptest::prelude::Strategy;
    use test_strategy::proptest;

    use crate::content::{IndexFrontmatter, LeafFrontmatter};

    #[derive(Debug)]
    struct Unexpected<T> {
        inner: T,
    }

    impl<T> std::fmt::Display for Unexpected<T>
    where
        T: std::fmt::Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // TODO is there a nicer way of displaying the value as falling back to the inner value's Display impl?
            write!(f, "Unexpected({:?})", self.inner)
        }
    }

    impl<T> std::error::Error for Unexpected<T> where T: std::fmt::Debug {}

    impl<T> From<T> for Unexpected<T> {
        fn from(value: T) -> Self {
            Self { inner: value }
        }
    }

    // TODO do these unit tests serve any purpose? These are more testing serde/toml than
    // anything else...

    #[proptest]
    fn leaf_frontmatter_deserializes_valid_str(
        #[strategy(
            MockLeafFrontmatter::builder().build()
        )]
        mock: MockLeafFrontmatter,
    ) {
        let toml = mock.try_to_toml().expect("mock should deserialize to toml");
        let res: Result<LeafFrontmatter, _> = toml::from_str(&toml);
        let _ = res.map_err(Unexpected::from)?;
    }

    #[proptest]
    fn leaf_frontmatter_rejects_missing_publication(
        #[strategy(
            MockLeafFrontmatter::builder().without_publication().build()
        )]
        mock: MockLeafFrontmatter,
    ) {
        let toml = mock.try_to_toml().expect("mock should deserialize to toml");
        let res: Result<LeafFrontmatter, _> = toml::from_str(&toml);

        match res {
            Err(_) => Ok(()),
            ok => Err(Unexpected::from(ok)),
        }?;
    }

    #[proptest]
    fn leaf_frontmatter_rejects_missing_title(
        #[strategy(
            MockLeafFrontmatter::builder().without_title().build()
        )]
        mock: MockLeafFrontmatter,
    ) {
        let toml = mock.try_to_toml().expect("mock should deserialize to toml");
        let res: Result<LeafFrontmatter, _> = toml::from_str(&toml);

        match res {
            Err(_) => Ok(()),
            ok => Err(Unexpected::from(ok)),
        }?;
    }

    #[proptest]
    fn leaf_frontmatter_rejects_invalid_publication(
        #[strategy(
            MockLeafFrontmatter::builder().with_publication("[a-z]{4}".into_defect().boxed()).build()
        )]
        mock: MockLeafFrontmatter,
    ) {
        let toml = mock.try_to_toml().expect("mock should deserialize to toml");
        let res: Result<LeafFrontmatter, _> = toml::from_str(&toml);

        match res {
            Err(_) => Ok(()),
            ok => Err(Unexpected::from(ok)),
        }?;
    }

    #[proptest]
    fn index_frontmatter_deserializes_valid_str(
        #[strategy(
            MockIndexFrontmatter::builder().build()
        )]
        mock: MockIndexFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let res: Result<IndexFrontmatter, _> = toml::from_str(&toml);
        let _ = res.map_err(Unexpected::from)?;
    }

    #[proptest]
    fn index_frontmatter_rejects_missing_publication(
        #[strategy(
            MockIndexFrontmatter::builder().without_publication().build()
        )]
        mock: MockIndexFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let res: Result<IndexFrontmatter, _> = toml::from_str(&toml);

        match res {
            Err(_) => Ok(()),
            ok => Err(Unexpected::from(ok)),
        }?;
    }

    #[proptest]
    fn index_frontmatter_rejects_missing_title(
        #[strategy(
            MockIndexFrontmatter::builder().without_title().build()
        )]
        mock: MockIndexFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let res: Result<IndexFrontmatter, _> = toml::from_str(&toml);

        match res {
            Err(_) => Ok(()),
            ok => Err(Unexpected::from(ok)),
        }?;
    }

    #[proptest]
    fn index_frontmatter_rejects_invalid_publication(
        #[strategy(
            MockIndexFrontmatter::builder().with_publication("[a-z]{4}".into_defect().boxed()).build()
        )]
        mock: MockIndexFrontmatter,
    ) {
        let toml = toml::to_string(&mock).expect("mock should deserialize to toml");
        let res: Result<IndexFrontmatter, _> = toml::from_str(&toml);

        match res {
            Err(_) => Ok(()),
            ok => Err(Unexpected::from(ok)),
        }?;
    }
}
