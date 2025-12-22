use crate::seal::Seal;

pub struct Index;

pub struct Leaf;

pub trait Mode: Seal {
    fn frontmatter_filestem() -> String;
    fn frontmatter_filename() -> String {
        Self::frontmatter_filestem() + ".toml"
    }
    fn typst_filename() -> String {
        Self::frontmatter_filestem() + ".typ"
    }
}

impl Seal for Index {}
impl Mode for Index {
    fn frontmatter_filestem() -> String {
        "index".to_owned()
    }
}

impl Seal for Leaf {}
impl Mode for Leaf {
    fn frontmatter_filestem() -> String {
        "page".to_owned()
    }
}
