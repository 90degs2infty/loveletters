#[derive(Debug)]
pub struct Unexpected<T> {
    inner: T,
}

impl<T> std::fmt::Display for Unexpected<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO is there a nicer way of displaying the value as falling back to the inner value's Display impl?
        write!(f, "unexpected value {:?}", self.inner)
    }
}

impl<T> std::error::Error for Unexpected<T> where T: std::fmt::Debug {}

impl<T> From<T> for Unexpected<T> {
    fn from(value: T) -> Self {
        Self { inner: value }
    }
}
