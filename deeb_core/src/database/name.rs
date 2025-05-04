#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct InstanceName(String);

impl From<&str> for InstanceName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
