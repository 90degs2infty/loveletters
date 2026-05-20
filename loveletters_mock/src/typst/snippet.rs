use proptest::prelude::*;

#[derive(Debug, Clone)]
pub struct Snippet(String);

impl Snippet {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_string(&self) -> &String {
        &self.0
    }
}

pub enum StrategyKind {
    Lorem,
    RandomText,
}

impl StrategyKind {
    pub fn into_strategy(&self) -> impl Strategy<Value = Snippet> {
        match self {
            Self::Lorem => Just(Snippet(String::from("#lorem(30)"))).boxed(),
            Self::RandomText => "[a-z]{30}".prop_map(Snippet).boxed(),
        }
    }
}
