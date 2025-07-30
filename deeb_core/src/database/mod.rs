use anyhow::Error;
use chrono::{DateTime, Utc};
use database_instance::DatabaseInstance;
use find_many_options::{FindManyOptions, FindManyOrder, OrderDirection};
use fs2::FileExt;
use instance_name::InstanceName;
use log::*;
use query::Query;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use ulid::Ulid;

use serde_json::{Map, Value, json};

use crate::entity::Entity;

pub mod database_instance;
pub mod find_many_options;
pub mod instance_name;
pub mod query;
pub mod transaction;
pub mod index;

pub type DbResult<T> = Result<T, anyhow::Error>;

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
    DroppedKey,
    AddedKey,
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
        find_many_options: Option<FindManyOptions>,
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
    DropKey {
        entity: Entity,
        key: String,
    },
    AddKey {
        entity: Entity,
        key: String,
        value: Value,
    },
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => a
            .as_f64()
            .partial_cmp(&b.as_f64())
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        _ => std::cmp::Ordering::Equal, // fallback for Null, Object, Array, etc.
    }
}

/// A database that stores multiple instances of data.
#[derive(Debug)]
pub struct Database {
    instances: HashMap<InstanceName, DatabaseInstance>,
}

impl Database {
    pub fn new() -> Self {
        let instances = HashMap::new();
        let database = Database { instances };
        database
    }

    pub fn add_instance(
        &mut self,
        name: &InstanceName,
        file_path: &str,
        entities: Vec<Entity>,
    ) -> Result<&mut Self, Error> {
        let instance = DatabaseInstance {
            file_path: file_path.to_string(),
            entities: entities.clone(),
            data: HashMap::new(),
        };
        self.instances.insert(name.clone(), instance);
        Ok(self)
    }

    pub fn load_instance(&mut self, name: &InstanceName) -> Result<&mut Self, Error> {
        let instance = self
            .instances
            .get_mut(name)
            .ok_or_else(|| Error::msg("Instance not found"))?;
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&instance.file_path);
        match file {
            Ok(mut file) => {
                file.lock_exclusive()?;
                let buf = &mut Vec::new();
                file.read_to_end(buf)?;
                instance.data = serde_json::from_slice(buf)?;
                fs2::FileExt::unlock(&file)?
            }
            Err(_) => {
                let mut file = fs::File::create(&instance.file_path)?;
                let entities = instance.entities.clone();
                let json = Value::Object(
                    entities
                        .iter()
                        .map(|entity| (entity.name.to_string().clone(), Value::Array(Vec::new())))
                        .collect(),
                );
                file.lock_exclusive()?;
                instance.data = serde_json::from_slice(serde_json::to_string(&json)?.as_bytes())?;
                file.write_all(serde_json::to_string(&json)?.as_bytes())?;
                file.sync_all()?;
                fs2::FileExt::unlock(&file)?
            }
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

    pub fn get_instance_name_by_entity(&self, entity: &Entity) -> Result<InstanceName, Error> {
        let name = self
            .instances
            .iter()
            .find(|(_, instance)| instance.entities.contains(entity))
            .map(|(name, _)| name);
        let name = name.ok_or_else(|| Error::msg("Can't Find Entity Name"))?;
        Ok(name.clone())
    }

    // Operations
    pub fn insert_one(&mut self, entity: &Entity, mut insert_value: Value) -> DbResult<Value> {
        // Check insert_value, it needs to be a JSON object.
        // It can not have field or `_id`.
        if !insert_value.is_object() {
            return Err(Error::msg("Value must be a JSON object"));
        }

        // Insert _id if it's not present
        let mut _id = None;
        if insert_value.get("_id").is_none() {
            _id = Some(Ulid::new());
            if let Some(obj) = insert_value.as_object_mut() {
                obj.insert("_id".to_string(), json!(_id.unwrap().to_string()));
            }
        }

        if insert_value.get("_created_at").is_none() {
            let server_time = if let Some(id) = _id {
                DateTime::<Utc>::from(id.datetime())
            } else {
                Utc::now()
            };

            if let Some(obj) = insert_value.as_object_mut() {
                obj.insert("_created_at".to_string(), json!(server_time.to_rfc3339()));
            }
        }

        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .entry(entity.name.clone())
            .or_insert(Vec::new());

        data.push(insert_value.clone());
        Ok(insert_value)
    }

    pub fn insert_many(
        &mut self,
        entity: &Entity,
        mut insert_values: Vec<Value>,
    ) -> DbResult<Vec<Value>> {
        for insert_value in insert_values.iter_mut() {
            if !insert_value.is_object() {
                return Err(Error::msg("Value must be a JSON object"));
            }
            // Insert _id if it's not present
            let mut _id = None;
            if insert_value.get("_id").is_none() {
                _id = Some(Ulid::new());
                if let Some(obj) = insert_value.as_object_mut() {
                    obj.insert("_id".to_string(), json!(_id.unwrap().to_string()));
                }
            }

            if insert_value.get("_created_at").is_none() {
                let server_time = if let Some(id) = _id {
                    DateTime::<Utc>::from(id.datetime())
                } else {
                    Utc::now()
                };

                if let Some(obj) = insert_value.as_object_mut() {
                    obj.insert("_created_at".to_string(), json!(server_time.to_rfc3339()));
                }
            }
        }
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .entry(entity.name.clone())
            .or_insert(Vec::new());

        let mut values = vec![];
        for insert_value in insert_values {
            data.push(insert_value.clone());
            values.push(insert_value);
        }
        Ok(values)
    }

    pub fn find_one(&self, entity: &Entity, query: Query) -> DbResult<Value> {
        let instance = self
            .get_instance_by_entity(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let result = data
            .iter()
            .find(|value| query.clone().matches(value).unwrap_or(false));
        result
            .map(|value| value.clone())
            .ok_or_else(|| Error::msg("Value not found"))
    }

    pub fn find_many(
        &self,
        entity: &Entity,
        query: Query,
        find_many_options: Option<FindManyOptions>,
    ) -> DbResult<Vec<Value>> {
        let instance = self
            .get_instance_by_entity(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let associated_entities = query.associated_entities();
        let FindManyOptions { skip, limit, order } = find_many_options.unwrap_or(FindManyOptions {
            skip: None,
            limit: None,
            order: None,
        });
        let mut data = data
            .iter()
            .map(|value| {
                let mut value = value.clone();
                for associated_entity in associated_entities.iter() {
                    let association = entity
                        .associations
                        .iter()
                        .find(|association| association.entity_name == associated_entity.name);

                    if association.is_none() {
                        continue;
                    }

                    let association = association.unwrap();
                    let association_query = Query::eq(
                        association.to.clone().as_str(),
                        value.get(association.from.clone()).unwrap().clone(), //TODO: Unwrap this
                                                                              //safely
                    );
                    let associated_data = self
                        .find_many(associated_entity, association_query, None)
                        .unwrap();

                    value.as_object_mut().unwrap().insert(
                        association.alias.clone().to_string(),
                        Value::Array(associated_data),
                    );
                }
                value
            })
            .collect::<Vec<Value>>();
        if let Some(ordering) = order {
            for FindManyOrder {
                property,
                direction,
            } in ordering.iter().rev()
            {
                data.sort_by(|a, b| {
                    let a_val = a.get(property).cloned().unwrap_or(Value::Null);
                    let b_val = b.get(property).cloned().unwrap_or(Value::Null);
                    let ord = compare_values(&a_val, &b_val);
                    match direction {
                        OrderDirection::Ascending => ord,
                        OrderDirection::Descending => ord.reverse(),
                    }
                });
            }
        }
        let result = data
            .iter()
            .filter(|value| query.clone().matches(value).unwrap_or(false));
        let skipped = result.skip(skip.unwrap_or(0) as usize);
        let limited = skipped.take(limit.unwrap_or(i32::MAX) as usize);
        Ok(limited.cloned().collect())
    }

    pub fn delete_one(&mut self, entity: &Entity, query: Query) -> DbResult<Value> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;
        let index = data
            .iter()
            .position(|value| query.clone().matches(value).unwrap_or(false))
            .ok_or_else(|| Error::msg("Value not found"))?;
        Ok(data.remove(index))
    }

    pub fn delete_many(&mut self, entity: &Entity, query: Query) -> DbResult<Vec<Value>> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(&entity.name)
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
    ) -> DbResult<Value> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(&entity.name)
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
                    _ => return Err(Error::msg("Update value must be a JSON object")),
                };
                let mut value = value.clone();
                for (update_key, update_value) in update_value {
                    if !update_value.is_null() {
                        value.insert(update_key, update_value);
                    }
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
    ) -> DbResult<Vec<Value>> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(&entity.name)
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
                        if !update_value.is_null() {
                            value.insert(update_key, update_value);
                        }
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

    pub fn commit(&self, names: Vec<InstanceName>) -> Result<(), Error> {
        for name in names {
            let instance = self
                .instances
                .get(&name)
                .ok_or_else(|| Error::msg("Instance not found"))?;

            // Convert the string path to PathBuf for manipulation
            let original_path = PathBuf::from(&instance.file_path);
            let mut tmp_path = original_path.clone();

            // Create a shadow file path like "campgrounds.json.tmp"
            tmp_path.set_extension("json.tmp");

            // Serialize the data
            let serialized = serde_json::to_vec(&instance.data)?;

            // Write to shadow file
            let mut tmp_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&tmp_path)
                .map_err(|e| {
                    error!("Failed to open temp path: {tmp_path:?}");
                    e
                })?;

            tmp_file.lock_exclusive()?;
            tmp_file.write_all(&serialized)?;
            tmp_file.sync_all()?;
            fs2::FileExt::unlock(&tmp_file)?;
            drop(tmp_file);

            // Atomically replace the original file with the shadow file
            std::fs::rename(&tmp_path, &original_path)?;
        }

        Ok(())
    }

    // Management
    pub fn drop_key(&mut self, entity: &Entity, key: &str) -> Result<(), Error> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;
        // Iterate through the entities
        for value in data.iter_mut() {
            match value {
                Value::Object(value) => {
                    if key.contains('.') {
                        let keys = key.split('.').collect::<Vec<&str>>();
                        let mut current = value.clone();
                        let mut key_exists = true;
                        for key in keys.iter().take(keys.len() - 1) {
                            current = match current.get_mut(*key) {
                                Some(Value::Object(current)) => current.clone(),
                                _ => {
                                    key_exists = false;
                                    break;
                                }
                            };
                        }
                        if key_exists {
                            let mut current = value;
                            for key in keys.iter().take(keys.len() - 1) {
                                current = match current.get_mut(*key) {
                                    Some(Value::Object(current)) => current,
                                    _ => {
                                        error!("Value must be a JSON object");
                                        return Err(Error::msg("Value must be a JSON object"));
                                    }
                                };
                            }
                            let key = keys.last().unwrap().to_owned();
                            current.remove(key);
                        } else {
                            continue;
                        }
                    } else {
                        value.remove(key);
                    }
                }
                _ => return Err(Error::msg("Value must be a JSON object")),
            }
        }
        Ok(())
    }

    pub fn add_key(
        &mut self,
        entity: &Entity,
        key: &str,
        default_value: Value,
    ) -> Result<(), Error> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get_mut(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;
        for current in data.iter_mut() {
            let keys = key.split('.').collect::<Vec<&str>>();
            let mut json = json!({});
            let mut current = current;
            for key in keys.iter().take(keys.len() - 1) {
                json.as_object_mut()
                    .unwrap()
                    .insert(key.to_string(), json!({}));
                let has_key = current.as_object().unwrap();
                if !has_key.contains_key(*key) || has_key.get(*key).unwrap().is_null() {
                    current
                        .as_object_mut()
                        .unwrap()
                        .insert(key.to_string(), json!({}));
                }
                current = current.get_mut(*key).unwrap();
            }
            let key = keys.last().unwrap().to_owned();
            if !current.is_object() {
                *current = Value::Object(Map::new());
            }
            current
                .as_object_mut()
                .unwrap()
                .insert(key.to_string(), default_value.clone());
        }
        Ok(())
    }
}
