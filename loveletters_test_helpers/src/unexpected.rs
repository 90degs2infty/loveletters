#[derive(Debug)]
pub struct Unexpected<T> {
    inner: T,
    pattern: String,
}

impl<T> std::fmt::Display for Unexpected<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO is there a nicer way of displaying the value as falling back to the inner value's Debug impl?
        write!(
            f,
            "unexpected value {:?} did not match pattern {}",
            self.inner, self.pattern
        )
    }
}

impl<T> std::error::Error for Unexpected<T> where T: std::fmt::Debug {}

impl<T> Unexpected<T> {
    pub fn new(value: T, pattern: String) -> Self {
        Self {
            inner: value,
            pattern,
        }
    }
}

#[macro_export]
macro_rules! into_unexpected {
    ( $e:expr , $pat:pat ) => {{
        match $e {
            $pat => Ok(()),
            r => Err(Unexpected::new(r, String::from(stringify!($pat)))),
        }
    }};
}
