use anyhow::Error;
use entity::Entity;
use name::Name;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use serde_json::Value;

pub mod entity;
pub mod name;

pub struct DatabaseInstance {
    file_path: String,
    entities: Vec<Entity>,
    data: HashMap<Entity, Vec<Value>>,
}

pub struct Database {
    instances: HashMap<Name, DatabaseInstance>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    /// Add a new instance to the database.
    pub fn add_instance(
        &mut self,
        name: Name,
        file_path: &str,
        entities: Vec<Entity>,
    ) -> &mut Self {
        self.instances.insert(
            name,
            DatabaseInstance {
                file_path: file_path.to_string(),
                entities,
                data: HashMap::new(),
            },
        );
        self
    }

    pub async fn load(&mut self) -> Result<&mut Self, Error> {
        for instance in self.instances.values_mut() {
            let mut file = tokio::fs::File::open(&instance.file_path).await?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await?;
            instance.data = serde_json::from_slice(&contents)?;
        }
        Ok(self)
    }

    pub fn get_instance_by_entity(&self, entity: &Entity) -> Option<&DatabaseInstance> {
        self.instances
            .values()
            .find(|instance| instance.entities.contains(entity))
    }

    pub fn get_instance_by_entity_mut(&mut self, entity: &Entity) -> Option<&mut DatabaseInstance> {
        self.instances
            .values_mut()
            .find(|instance| instance.entities.contains(entity))
    }

    // Operations
    pub async fn insert(&mut self, entity: &Entity, insert_value: Value) -> Result<Value, Error> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance.data.entry(entity.clone()).or_insert(Vec::new());
        data.push(insert_value.clone());
        Ok(insert_value)
    }

    pub async fn find_one(&self, entity: &Entity, query: Value) -> Result<Value, Error> {
        let instance = self
            .get_instance_by_entity(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get(entity)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let result = data.iter().find(|value| {
            // Find the value that matches the query
            query
                .as_object()
                .map(|query| {
                    query
                        .iter()
                        .all(|(key, value)| value == value.get(key).unwrap())
                })
                .unwrap_or(false)
        });
        result
            .map(|value| value.clone())
            .ok_or_else(|| Error::msg("Value not found"))
    }

    pub async fn commit(&self, name: Name) -> Result<&Self, Error> {
        let instance = self
            .instances
            .get(&name)
            .ok_or_else(|| Error::msg("Instance not found"))?;
        let mut file = tokio::fs::File::create(&instance.file_path).await?;
        file.write_all(serde_json::to_string(&instance.data)?.as_bytes())
            .await?;
        Ok(self)
    }
}
