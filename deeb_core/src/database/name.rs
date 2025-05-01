#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Name(String);

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
