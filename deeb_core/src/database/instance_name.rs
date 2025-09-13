use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct InstanceName(pub String);

impl From<&str> for InstanceName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl Display for InstanceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
