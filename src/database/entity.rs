use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct Entity(String);

impl From<&str> for Entity {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
