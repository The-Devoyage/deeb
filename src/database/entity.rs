use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct EntityName(pub String);

impl From<&str> for EntityName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for EntityName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: EntityName,
}

impl Entity {
    pub fn new(s: &str) -> Self {
        Entity {
            name: EntityName::from(s),
        }
    }
}
