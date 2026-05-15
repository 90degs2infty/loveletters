use lattice::{IntoCorrect, Site, So};
use proptest::prelude::*;
use serde::Serialize;
use time::{Date, UtcDateTime};

// TODO replace strings with COW semantics to make cloneing cheap

#[derive(Debug, Clone, Serialize)]
pub struct Title(String);

impl Title {
    pub fn prop_valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9 ]*".prop_map(Title)
    }
}

#[derive(Debug, Serialize)]
pub struct Frontmatter {
    #[serde(skip_serializing_if = "So::is_vacant")]
    publication: So<Date, String>,
    #[serde(skip_serializing_if = "So::is_vacant")]
    title: So<Title, String>,
}

impl Frontmatter {
    pub fn builder() -> FrontmatterStrategyBuilder {
        FrontmatterStrategyBuilder::valid()
    }
}

pub struct FrontmatterStrategyBuilder {
    publication: BoxedStrategy<So<Date, String>>,
    title: BoxedStrategy<So<Title, String>>,
}

impl FrontmatterStrategyBuilder {
    pub fn valid() -> Self {
        Self {
            publication: (UtcDateTime::MIN.unix_timestamp()..=UtcDateTime::MAX.unix_timestamp())
                .prop_map(|timestamp| {
                    UtcDateTime::from_unix_timestamp(timestamp)
                        .expect("timestamp in [MIN, MAX] should be valid")
                        .date()
                })
                .into_correct()
                .boxed(),
            title: Title::prop_valid().into_correct().boxed(),
        }
    }

    pub fn with_publication(&mut self, publication: BoxedStrategy<So<Date, String>>) -> &mut Self {
        self.publication = publication;
        self
    }

    pub fn without_publication(&mut self) -> &mut Self {
        self.publication = Site::prop_vacant().boxed();
        self
    }

    pub fn with_title(&mut self, title: BoxedStrategy<So<Title, String>>) -> &mut Self {
        self.title = title;
        self
    }

    pub fn without_title(&mut self) -> &mut Self {
        self.title = Site::prop_vacant().boxed();
        self
    }

    pub fn build(&mut self) -> impl Strategy<Value = Frontmatter> + use<> {
        let Self { publication, title } = self;
        (publication.clone(), title.clone())
            .prop_map(|(publication, title)| Frontmatter { publication, title })
    }
}
