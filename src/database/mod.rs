use anyhow::Error;
use entity::Entity;
use fs2::FileExt;
use name::Name;
use query::Query;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};

use serde_json::Value;

pub mod entity;
pub mod name;
pub mod query;
pub mod transaction;

/// A database instance. Tpically, a database instance is a JSON file on disk.
/// The `entities` field is a list of entities that are stored in the database used
/// by Deeb to index the data.
pub struct DatabaseInstance {
    file_path: String,
    entities: Vec<Entity>,
    data: HashMap<Entity, Vec<Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutedValue {
    InsertedOne(Value),
    InsertedMany(Vec<Value>),
    FoundOne,
    FoundMany,
    DeletedOne(Value),
    DeletedMany(Vec<Value>),
    UpdatedOne(Value),
    UpdatedMany(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    InsertOne {
        entity: Entity,
        value: Value,
    },
    InsertMany {
        entity: Entity,
        values: Vec<Value>,
    },
    FindOne {
        entity: Entity,
        query: Query,
    },
    FindMany {
        entity: Entity,
        query: Query,
    },
    DeleteOne {
        entity: Entity,
        query: Query,
    },
    DeleteMany {
        entity: Entity,
        query: Query,
    },
    UpdateOne {
        entity: Entity,
        query: Query,
        value: Value,
    },
    UpdateMany {
        entity: Entity,
        query: Query,
        value: Value,
    },
}

/// A database that stores multiple instances of data.
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

    pub fn load_instance(&mut self, name: &Name) -> Result<&mut Self, Error> {
        let instance = self
            .instances
            .get_mut(name)
            .ok_or_else(|| Error::msg("Instance not found"))?;
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&instance.file_path)?;
        file.lock_exclusive()?;
        let buf = &mut Vec::new();
        file.read_to_end(buf)?;
        instance.data = serde_json::from_slice(buf)?;
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

    pub fn get_instance_name_by_entity(&self, entity: &Entity) -> Result<Name, Error> {
        let name = self
            .instances
            .iter()
            .find(|(_, instance)| instance.entities.contains(entity))
            .map(|(name, _)| name);
        let name = name.ok_or_else(|| Error::msg("Entity not found"))?;
        Ok(name.clone())
    }

    // Operations
    pub fn insert(&mut self, entity: &Entity, insert_value: Value) -> Result<Value, Error> {
        // Check insert_value, it needs to be a JSON object.
        // It can not have field or `_id`.
        if !insert_value.is_object() {
            return Err(Error::msg("Value must be a JSON object"));
        }
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance.data.entry(entity.clone()).or_insert(Vec::new());

        data.push(insert_value.clone());
        Ok(insert_value)
    }

    pub fn insert_many(
        &mut self,
        entity: &Entity,
        insert_values: Vec<Value>,
    ) -> Result<Vec<Value>, Error> {
        for insert_value in insert_values.iter() {
            if !insert_value.is_object() {
                return Err(Error::msg("Value must be a JSON object"));
            }
        }
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance.data.entry(entity.clone()).or_insert(Vec::new());

        let mut values = vec![];
        for insert_value in insert_values {
            data.push(insert_value.clone());
            values.push(insert_value);
        }
        Ok(values)
    }

    pub fn find_one(&self, entity: &Entity, query: Query) -> Result<Value, Error> {
        let instance = self
            .get_instance_by_entity(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get(entity)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let result = data
            .iter()
            .find(|value| query.clone().matches(value).unwrap_or(false));
        result
            .map(|value| value.clone())
            .ok_or_else(|| Error::msg("Value not found"))
    }

    pub fn find_many(&self, entity: &Entity, query: Query) -> Result<Vec<Value>, Error> {
        let instance = self
            .get_instance_by_entity(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get(entity)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let result = data
            .iter()
            .filter(|value| query.clone().matches(value).unwrap_or(false));
        Ok(result.cloned().collect())
    }

    pub fn delete_one(&mut self, entity: &Entity, query: Query) -> Result<Value, Error> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(entity)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let index = data
            .iter()
            .position(|value| query.clone().matches(value).unwrap_or(false))
            .ok_or_else(|| Error::msg("Value not found"))?;
        Ok(data.remove(index))
    }

    pub fn delete_many(&mut self, entity: &Entity, query: Query) -> Result<Vec<Value>, Error> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(entity)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let indexes = data
            .iter()
            .enumerate()
            .filter(|(_, value)| query.clone().matches(value).unwrap_or(false))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let mut values = vec![];
        for index in indexes.iter().rev() {
            values.push(data.remove(*index));
        }
        Ok(values)
    }

    pub fn update_one(
        &mut self,
        entity: &Entity,
        query: Query,
        update_value: Value,
    ) -> Result<Value, Error> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(entity)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let index = data
            .iter()
            .position(|value| query.clone().matches(value).unwrap_or(false))
            .ok_or_else(|| Error::msg("Value not found"))?;
        let value = data
            .get_mut(index)
            .ok_or_else(|| Error::msg("Value not found"))?;
        // combine the values together, so that the updated values are merged with the existing values.
        let new_value = match value {
            Value::Object(value) => {
                let update_value = match update_value {
                    Value::Object(update_value) => update_value,
                    _ => return Err(Error::msg("Value must be a JSON object")),
                };
                let mut value = value.clone();
                for (update_key, update_value) in update_value {
                    value.insert(update_key, update_value);
                }
                Value::Object(value)
            }
            _ => return Err(Error::msg("Value must be a JSON object")),
        };
        *value = new_value.clone();
        Ok(new_value)
    }

    pub fn update_many(
        &mut self,
        entity: &Entity,
        query: Query,
        update_value: Value,
    ) -> Result<Vec<Value>, Error> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(entity)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let indexes = data
            .iter()
            .enumerate()
            .filter(|(_, value)| query.clone().matches(value).unwrap_or(false))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let mut values = vec![];
        for index in indexes.iter() {
            let value = data
                .get_mut(*index)
                .ok_or_else(|| Error::msg("Value not found"))?;
            // combine the values together, so that the updated values are merged with the existing values.
            let new_value = match value {
                Value::Object(value) => {
                    let update_value = match update_value.clone() {
                        Value::Object(update_value) => update_value,
                        _ => return Err(Error::msg("Value must be a JSON object")),
                    };
                    let mut value = value.clone();
                    for (update_key, update_value) in update_value {
                        value.insert(update_key, update_value);
                    }
                    Value::Object(value)
                }
                _ => return Err(Error::msg("Value must be a JSON object")),
            };
            *value = new_value.clone();
            values.push(new_value);
        }
        Ok(values)
    }

    pub fn commit(&self, name: Vec<Name>) -> Result<(), Error> {
        for name in name {
            let instance = self
                .instances
                .get(&name)
                .ok_or_else(|| Error::msg("Instance not found"))?;
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&instance.file_path)?;
            file.lock_exclusive()?;
            file.set_len(0)?;
            file.write_all(serde_json::to_string(&instance.data)?.as_bytes())?;
            file.unlock()?;
        }
        Ok(())
    }
}
