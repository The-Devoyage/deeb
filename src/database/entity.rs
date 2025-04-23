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
pub struct EntityAssociation {
    pub from: String,
    pub to: String,
    pub entity_name: EntityName,
    /// Uses the entity name as the alias if not provided.
    pub alias: EntityName,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: EntityName,
    pub primary_key: Option<String>,
    pub associations: Vec<EntityAssociation>,
    pub indexes: Vec<Index>,
}

impl Entity {
    /// Create a new entity.
    /// # Example
    /// ```rust
    /// use deeb::*;
    /// let user = Entity::new("user");
    /// ```
    pub fn new(s: &str) -> Self {
        Entity {
            name: EntityName::from(s),
            primary_key: None,
            associations: vec![],
            indexes: vec![],
        }
    }

    pub fn primary_key(&mut self, key: &str) -> Self {
        self.primary_key = Some(key.to_string());
        self.clone()
    }

    pub fn add_index(&mut self, name: &str, columns: Vec<&str>) -> &mut Self {
        self.indexes.push(Index {
            name: name.to_string(),
            columns: columns.iter().map(|c| c.to_string()).collect(),
        });
        self
    }

    pub fn associate<'a, N>(
        &mut self,
        entity: &'a mut Entity,
        from: &str,
        alias: Option<N>,
    ) -> Result<Self, String>
    where
        N: Into<EntityName>,
    {
        // Make sure the keys are valid by enforcing primary keys in both entities
        if self.primary_key.is_none() {
            return Err(format!(
                "Origin Entity `{}` does not have a primary key.",
                self.name
            ));
        }

        if entity.primary_key.is_none() {
            return Err(format!(
                "Associated Entity `{}` does not have a primary key.",
                entity.name
            ));
        }

        let alias = alias.map_or_else(|| entity.name.clone(), |a| a.into());

        self.associations.push(EntityAssociation {
            from: entity.primary_key.clone().unwrap(),
            to: from.to_string(),
            entity_name: entity.name.clone(),
            alias,
        });

        entity.associations.push(EntityAssociation {
            from: from.to_string(),
            to: entity.primary_key.clone().unwrap(),
            entity_name: self.name.clone(),
            alias: self.name.clone(),
        });

        Ok(self.clone())
    }
}
