use std::{fs, path::Path};

use proptest::{
    prelude::{Arbitrary, BoxedStrategy, Just, Strategy},
    prop_oneof,
};
use serde::Serialize;
use time::OffsetDateTime;

// We allow everything except for characters triggering special processing from within markup.
//
// See the list at https://typst.app/docs/reference/syntax/ and also keep in mind shorthands at
// https://typst.app/docs/reference/symbols/
// const BASIC_MARKUP_STRATEGY: &str = "[a-zA-Z0-9 ]*";
const BASIC_MARKUP_STRATEGY: &str = "[^#$\\[\\]\\*_`<>@=-\\\\'\"]*";

fn title() -> impl Strategy<Value = String> {
    BASIC_MARKUP_STRATEGY
}

fn publication() -> impl Strategy<Value = OffsetDateTime> {
    (-377705116800i64..253402300799i64)
        .prop_map(|ts| OffsetDateTime::from_unix_timestamp(ts).expect("range -377705116800i64..253402300799i64 should have contained valid unix timestamps only"))
}

#[derive(Debug, Clone, Serialize)]
pub struct LeafFrontmatter {
    title: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::iso8601::option"
    )]
    publication: Option<OffsetDateTime>,
}

impl LeafFrontmatter {
    pub fn missing_title() -> impl Strategy<Value = Self> {
        publication().prop_map(|publication| LeafFrontmatter {
            title: None,
            publication: Some(publication),
        })
    }

    pub fn missing_publication() -> impl Strategy<Value = Self> {
        title().prop_map(|title| LeafFrontmatter {
            title: Some(title),
            publication: None,
        })
    }

    pub fn missing_everything() -> impl Strategy<Value = Self> {
        Just(LeafFrontmatter {
            title: None,
            publication: None,
        })
    }

    pub fn invalid() -> impl Strategy<Value = Self> {
        prop_oneof![
            Self::missing_title(),
            Self::missing_publication(),
            Self::missing_everything()
        ]
    }

    pub fn valid() -> impl Strategy<Value = Self> {
        (title(), publication()).prop_map(|(title, publication)| LeafFrontmatter {
            title: Some(title),
            publication: Some(publication),
        })
    }

    pub fn write_toml(&self, path: &Path) {
        let toml = toml::to_string_pretty(self).unwrap();
        fs::write(path, toml).unwrap()
    }
}

impl Arbitrary for LeafFrontmatter {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}
