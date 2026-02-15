use std::{collections::HashMap, fmt, fs, path::Path};

use proptest::{
    collection::{hash_map, vec},
    prelude::{Arbitrary, BoxedStrategy, Just, Strategy, any},
    prop_oneof,
};
use serde::Serialize;
use time::OffsetDateTime;
use url::Url as RawUrl;

fn write_toml<V: Serialize>(value: &V, path: &Path) {
    let toml = toml::to_string_pretty(value).unwrap();
    fs::write(path, toml).unwrap()
}

// We allow everything except for characters triggering special processing from within markup.
//
// See the list at https://typst.app/docs/reference/syntax/ and also keep in mind shorthands at
// https://typst.app/docs/reference/symbols/
const BASIC_MARKUP_STRATEGY: &str = "[a-zA-Z0-9 ]*";
// const BASIC_MARKUP_STRATEGY: &str = "[^#$\\[\\]\\*_`<>@=-\\\\'\"]*";

fn title() -> impl Strategy<Value = String> {
    BASIC_MARKUP_STRATEGY
}

fn publication() -> impl Strategy<Value = OffsetDateTime> {
    (-377705116800i64..253402300799i64)
        .prop_map(|ts| OffsetDateTime::from_unix_timestamp(ts).expect("range -377705116800i64..253402300799i64 should have contained valid unix timestamps only"))
}

fn author() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]"
}

#[derive(Debug, Clone)]
pub struct TypstFile {
    content: String,
}

impl TypstFile {
    pub fn valid() -> impl Strategy<Value = Self> {
        BASIC_MARKUP_STRATEGY.prop_map(|content| TypstFile { content })
    }

    pub fn write_typ(&self, path: &Path) {
        fs::write(path, &self.content).unwrap()
    }
}

impl Arbitrary for TypstFile {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct LeafFrontmatter {
    title: Option<String>,
    #[serde(with = "time::serde::iso8601::option")]
    publication: Option<OffsetDateTime>,
}

impl LeafFrontmatter {
    pub fn missing_title() -> impl Strategy<Value = Self> {
        publication().prop_map(|publication| Self {
            title: None,
            publication: Some(publication),
        })
    }

    pub fn missing_publication() -> impl Strategy<Value = Self> {
        title().prop_map(|title| Self {
            title: Some(title),
            publication: None,
        })
    }

    pub fn invalid() -> impl Strategy<Value = Self> {
        prop_oneof![Self::missing_title(), Self::missing_publication(),]
    }

    pub fn valid() -> impl Strategy<Value = Self> {
        (title(), publication()).prop_map(|(title, publication)| Self {
            title: Some(title),
            publication: Some(publication),
        })
    }

    pub fn write_toml(&self, path: &Path) {
        write_toml(self, path);
    }
}

impl Arbitrary for LeafFrontmatter {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexFrontmatter {
    title: Option<String>,
    #[serde(with = "time::serde::iso8601::option")]
    publication: Option<OffsetDateTime>,
}

impl IndexFrontmatter {
    pub fn missing_title() -> impl Strategy<Value = Self> {
        publication().prop_map(|publication| Self {
            title: None,
            publication: Some(publication),
        })
    }

    pub fn missing_publication() -> impl Strategy<Value = Self> {
        title().prop_map(|title| Self {
            title: Some(title),
            publication: None,
        })
    }

    pub fn invalid() -> impl Strategy<Value = Self> {
        prop_oneof![Self::missing_title(), Self::missing_publication(),]
    }

    pub fn valid() -> impl Strategy<Value = Self> {
        (title(), publication()).prop_map(|(title, publication)| Self {
            title: Some(title),
            publication: Some(publication),
        })
    }

    pub fn write_toml(&self, path: &Path) {
        write_toml(self, path);
    }
}

impl Arbitrary for IndexFrontmatter {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}

#[derive(Debug, Clone)]
pub struct Page<F> {
    frontmatter: Option<F>,
    root_file: Option<TypstFile>,
}

impl<F> Page<F>
where
    F: fmt::Debug,
{
    pub fn general(
        frontmatter: impl Strategy<Value = Option<F>>,
        root_file: impl Strategy<Value = Option<TypstFile>>,
    ) -> impl Strategy<Value = Self> {
        (frontmatter, root_file).prop_map(|(frontmatter, root_file)| Self {
            frontmatter: frontmatter,
            root_file: root_file,
        })
    }
}

impl<F> Page<F>
where
    F: Arbitrary,
{
    pub fn valid() -> impl Strategy<Value = Self> {
        Self::general(
            any::<F>().prop_map(Option::Some),
            TypstFile::valid().prop_map(Option::Some),
        )
    }
}

impl<F> Page<F>
where
    F: Serialize,
{
    pub fn write_to_dir(&self, dir: &Path) {
        let Self {
            frontmatter,
            root_file,
        } = self;

        if let Some(frontmatter) = frontmatter {
            write_toml(&frontmatter, &dir.join("page.toml"));
        }

        if let Some(root_file) = root_file {
            root_file.write_typ(&dir.join("page.typ"));
        }
    }
}

impl<F> Arbitrary for Page<F>
where
    F: Arbitrary + 'static,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}

pub type LeafPage = Page<LeafFrontmatter>;
pub type IndexPage = Page<IndexFrontmatter>;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Slug(String);

impl Slug {
    pub fn valid() -> impl Strategy<Value = Self> {
        "[a-zA-Z0-9]+".prop_map(|s| Self(s))
    }
}

impl Arbitrary for Slug {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    index: Option<IndexPage>,
    pages: HashMap<Slug, LeafPage>,
    sub_sections: HashMap<Slug, Section>,
}

impl Section {
    pub fn valid() -> impl Strategy<Value = Self> {
        // Inspired by the book: https://proptest-rs.github.io/proptest/proptest/tutorial/recursive.html
        //
        // Build a strategy for a leaf section (that is, a section without children sections) first
        // and use prop_recursive to build the recursive structure.

        // A note on keys: luckily, we do not have to take care of unique slugs. HashMap will
        // implicitly take care of non-unique slugs for pages and subsections.
        // TODO: this is only true for pages and subsections by themselves. However, there can be
        // collisions between pages and subsections on write to disk!
        let index = IndexPage::valid().prop_map(Option::Some);
        let pages = hash_map(Slug::valid(), LeafPage::valid(), 0..10);
        let no_children = Just(HashMap::new());
        let leaf_section =
            (index, pages, no_children).prop_map(|(index, pages, sub_sections)| Self {
                index,
                pages,
                sub_sections,
            });

        leaf_section
            .prop_recursive(4, 16, 4, |element| {
                let index = IndexPage::valid().prop_map(Option::Some);
                let pages = hash_map(Slug::valid(), LeafPage::valid(), 0..10);

                let children_sections =
                    vec((Slug::valid(), element), 0..4).prop_map(HashMap::from_iter);
                (index, pages, children_sections).prop_map(|(index, pages, children)| Self {
                    index,
                    pages,
                    sub_sections: children,
                })
            })
            .boxed()
    }

    pub fn write_to_dir(&self, path: &Path) {
        let Self {
            index,
            pages,
            sub_sections,
        } = self;

        if let Some(index) = index {
            let out_dir = path.join("_index");
            fs::create_dir(&out_dir).unwrap();
            index.write_to_dir(&out_dir);
        }

        for (s, p) in pages {
            let out_dir = path.join(&s.0);
            fs::create_dir(&out_dir).unwrap();
            p.write_to_dir(&out_dir);
        }

        for (s, sub) in sub_sections {
            let out_dir = path.join(&s.0);
            fs::create_dir(&out_dir).unwrap();
            sub.write_to_dir(&out_dir);
        }
    }

    pub fn num_leafs(&self) -> usize {
        self.pages.len()
            + self
                .sub_sections
                .values()
                .fold(0, |acc, sec| acc + sec.num_leafs())
    }

    // Boxing is required to make types non-recursive and pass the compiler
    pub fn leafs(&self) -> Box<dyn Iterator<Item = &LeafPage> + '_> {
        Box::new(
            self.pages
                .values()
                .chain(self.sub_sections.values().map(Self::leafs).flatten()),
        )
    }

    pub fn leaf_at(&self, idx: usize) -> Option<&LeafPage> {
        self.leafs().skip(idx).next()
    }

    // Boxing is required to make types non-recursive and pass the compiler
    pub fn leafs_mut(&mut self) -> Box<dyn Iterator<Item = &mut LeafPage> + '_> {
        Box::new(
            self.pages.values_mut().chain(
                self.sub_sections
                    .values_mut()
                    .map(Self::leafs_mut)
                    .flatten(),
            ),
        )
    }

    pub fn leaf_at_mut(&mut self, idx: usize) -> Option<&mut LeafPage> {
        self.leafs_mut().skip(idx).next()
    }

    // /// Number of (sub-)sections including `self`.
    // pub fn num_sections(&self) -> usize {
    //     1 + self
    //         .sub_sections
    //         .values()
    //         .fold(0, |acc, sec| acc + sec.num_sections())
    // }

    // /// The first item in the returned iterator yields this [`Section`] itself.
    // ///
    // /// No guarantees on the order of traversal.
    // pub fn sections(&self) -> impl Iterator<Item = &Self> {
    //     once(self).chain(self.sub_sections.values().map(Self::sections).flatten())
    // }

    // /// Subsections contained in this [`Section`].
    // ///
    // /// Note that in contrast to [`Section::sections`], this [`Section`] itself is not yielded by
    // /// the returned iterator.
    // ///
    // /// No guarantees on the order of traversal.
    // pub fn subsections_mut(&mut self) -> impl Iterator<Item = &mut Self> {
    //     self.sub_sections
    //         .values_mut()
    //         .map(Self::subsections_mut)
    //         .flatten()
    // }

    // pub fn section_at(&self, idx: usize) -> Option<&Self> {
    //     self.sections().skip(idx).next()
    // }

    // pub fn section_at_mut(&mut self, idx: usize) -> Option<&mut Self> {
    //     if idx == 0 {
    //         return Some(self);
    //     }

    //     self.subsections_mut().skip(idx - 1).next()
    // }
}

impl Arbitrary for Section {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}

#[derive(Debug, Clone, Serialize)]
struct Url(RawUrl);

impl Url {
    fn scheme() -> impl Strategy<Value = String> {
        prop_oneof![Just(String::from("http")), Just(String::from("https")),]
    }

    fn toplevel_domain() -> impl Strategy<Value = String> {
        prop_oneof![
            Just(String::from("org")),
            Just(String::from("com")),
            Just(String::from("io")),
            Just(String::from("dev")),
        ]
    }

    fn secondlevel_domain() -> impl Strategy<Value = String> {
        "[a-z0-9]+"
    }

    pub fn valid() -> impl Strategy<Value = Self> {
        (
            Self::scheme(),
            Self::secondlevel_domain(),
            Self::toplevel_domain(),
        )
            .prop_map(|(scheme, snd, top)| {
                Self(
                    RawUrl::parse(&(scheme + &snd + &top))
                        .expect("parts should have resembled a valid Url"),
                )
            })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfig {
    title: Option<String>,
    author: Option<String>,
    root: Option<Url>,
}

impl ProjectConfig {
    pub fn missing_title() -> impl Strategy<Value = Self> {
        (author(), Url::valid()).prop_map(|(author, url)| Self {
            title: None,
            author: Some(author),
            root: Some(url),
        })
    }

    pub fn missing_author() -> impl Strategy<Value = Self> {
        (title(), Url::valid()).prop_map(|(title, url)| Self {
            title: Some(title),
            author: None,
            root: Some(url),
        })
    }

    pub fn missing_root() -> impl Strategy<Value = Self> {
        (title(), author()).prop_map(|(title, author)| Self {
            title: Some(title),
            author: Some(author),
            root: None,
        })
    }

    pub fn invalid() -> impl Strategy<Value = Self> {
        prop_oneof![
            Self::missing_title(),
            Self::missing_author(),
            Self::missing_root()
        ]
    }

    pub fn valid() -> impl Strategy<Value = Self> {
        (title(), author(), Url::valid()).prop_map(|(title, author, url)| Self {
            title: Some(title),
            author: Some(author),
            root: Some(url),
        })
    }

    pub fn write_toml(&self, path: &Path) {
        write_toml(self, path);
    }
}

impl Arbitrary for ProjectConfig {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    content: Option<Section>,
    config: Option<ProjectConfig>,
}

impl Project {
    fn general_helper(
        content: impl Strategy<Value = Option<Section>>,
        config: impl Strategy<Value = Option<ProjectConfig>>,
    ) -> impl Strategy<Value = Self> {
        (content, config).prop_map(|(content, config)| Self { content, config })
    }

    pub fn general(
        content: impl Strategy<Value = Section>,
        config: impl Strategy<Value = ProjectConfig>,
    ) -> impl Strategy<Value = Self> {
        Self::general_helper(
            content.prop_map(Option::Some),
            config.prop_map(Option::Some),
        )
    }

    pub fn missing_content() -> impl Strategy<Value = Self> {
        Self::general_helper(Just(None), ProjectConfig::valid().prop_map(Option::Some))
    }

    pub fn missing_config() -> impl Strategy<Value = Self> {
        Self::general_helper(Section::valid().prop_map(Option::Some), Just(None))
    }

    pub fn missing_something() -> impl Strategy<Value = Self> {
        prop_oneof![Self::missing_config(), Self::missing_content()]
    }

    pub fn valid() -> impl Strategy<Value = Self> {
        Self::general(Section::valid(), ProjectConfig::valid())
    }

    pub fn write_to_dir(&self, path: &Path) {
        let Self { content, config } = self;

        if let Some(content) = content {
            let content_dir = path.join("content");
            fs::create_dir(&content_dir).unwrap();
            content.write_to_dir(&content_dir);
        }

        if let Some(config) = config {
            let config_file = path.join("loveletters.toml");
            config.write_toml(&config_file);
        }
    }
}

impl Arbitrary for Project {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::valid().boxed()
    }
}
