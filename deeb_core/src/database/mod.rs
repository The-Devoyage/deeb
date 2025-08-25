use anyhow::Error;
use chrono::{DateTime, Utc};
use database_instance::{DatabaseInstance, PrimaryKeyValue};
use find_many_options::{FindManyOptions, FindManyOrder, OrderDirection};
use fs2::FileExt;
use index_constrant::{collect_constraints, query_with_index};
use instance_name::InstanceName;
use log::*;
use query::{Key, Query};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use ulid::Ulid;

use serde_json::{Map, Value, json};

use crate::entity::{Entity, EntityName};

pub mod database_instance;
pub mod find_many_options;
pub mod index;
pub mod index_constrant;
pub mod instance_name;
pub mod query;
pub mod transaction;

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
            indexes: HashMap::new(),
        };
        self.instances.insert(name.clone(), instance);
        Ok(self)
    }

    fn initialize_empty_data(
        entities: &Vec<Entity>,
    ) -> HashMap<EntityName, HashMap<String, Value>> {
        entities
            .iter()
            .map(|entity| (entity.name.clone(), HashMap::new()))
            .collect()
    }

    pub fn load_instance(&mut self, name: &InstanceName) -> DbResult<&mut Self> {
        let instance = self
            .instances
            .get_mut(name)
            .ok_or_else(|| Error::msg("Instance not found"))?;

        let file_result = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&instance.file_path);

        match file_result {
            Ok(mut file) => {
                file.lock_exclusive()?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;

                if buf.is_empty() {
                    instance.data = Database::initialize_empty_data(&instance.entities);
                } else {
                    instance.data = serde_json::from_slice(&buf).map_err(|e| {
                        log::error!("Failed to read json.");
                        e
                    })?;
                }

                fs2::FileExt::unlock(&file)?
            }
            Err(_) => {
                let mut file = fs::File::create(&instance.file_path)?;
                file.lock_exclusive()?;

                let data = Database::initialize_empty_data(&instance.entities);
                let json = serde_json::to_string(&data)?;
                file.write_all(json.as_bytes())?;
                file.sync_all()?;

                instance.data = data;
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
        let data = instance.get_or_init(&entity.name);

        let primary_key_value = PrimaryKeyValue::new(&insert_value, &entity.primary_key)?;

        data.insert(primary_key_value.to_string(), insert_value.clone());

        //TODO: Need to update built index with the custom indexes

        // Handle indexing
        self.append_indexes(entity, &[insert_value.clone()])?;

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

        // Do one mutable borrow of self to insert all values.
        {
            let instance = self
                .get_instance_by_entity_mut(entity)
                .ok_or_else(|| Error::msg("Entity not found"))?;
            let data = instance.get_or_init(&entity.name);

            for insert_value in &insert_values {
                let primary_key_value = PrimaryKeyValue::new(insert_value, &entity.primary_key)?;
                data.insert(primary_key_value.to_string(), insert_value.clone());
            }
            //TODO: Need to index the custom indexes
        }

        // Append indexes in a separate borrow
        self.append_indexes(entity, &insert_values)?;

        Ok(insert_values)
    }

    pub fn find_one(&self, entity: &Entity, query: Query) -> DbResult<Value> {
        let instance = self
            .get_instance_by_entity(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;

        // Collect constraints for index use
        let mut constraints = HashMap::new();
        collect_constraints(&query, &mut constraints);

        // 1. Try indexed search first
        if let Some(index_store) = instance.indexes.get(&entity.name) {
            if !constraints.is_empty() {
                for idx in &index_store.indexes {
                    if let Some(results) = query_with_index(idx, &constraints) {
                        for id in results {
                            if let Some(value) = data.get(&id) {
                                if query.matches(value).unwrap_or(false) {
                                    let mut found = value.clone();
                                    self.apply_associations(&mut found, &query, entity);
                                    return Ok(found);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Fallback: linear scan
        for value in data.values() {
            if query.matches(value).unwrap_or(false) {
                let mut found = value.clone();
                self.apply_associations(&mut found, &query, entity);
                return Ok(found);
            }
        }

        Err(Error::msg("Value not found"))
    }

    fn search_with_indexes<'a>(
        &'a self,
        entity: &Entity,
        query: &Query,
    ) -> DbResult<Vec<&'a Value>> {
        let instance = self
            .get_instance_by_entity(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;
        let data = instance
            .data
            .get(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;

        // Gather constraints
        let mut constraints = HashMap::new();
        collect_constraints(query, &mut constraints);

        // 1. Try indexed search first
        if let Some(index_store) = instance.indexes.get(&entity.name) {
            println!("INDEX");
            if !constraints.is_empty() {
                println!("CONSTRAINTS FOUND");
                for idx in &index_store.indexes {
                    println!("IDX: {idx:?}");
                    if let Some(results) = query_with_index(idx, &constraints) {
                        let matches: Vec<&Value> = results
                            .into_iter()
                            .filter_map(|id| data.get(&id))
                            .filter(|v| query.matches(v).unwrap_or(false))
                            .collect();
                        if !matches.is_empty() {
                            return Ok(matches);
                        }
                    }
                }
            }
        }

        // 2. Fallback: full scan
        // TODO: We are falling back to early. In the event that the index query does not match
        // anyhthing but in the example that we are searching associated entities - We don't yet
        // have that data?!
        // but we also dont want to find every association for every record right?
        println!("FULL SCAN");
        let matches: Vec<&Value> = data
            .values()
            .filter(|v| query.matches(v).unwrap_or(false))
            .collect();

        Ok(matches)
    }

    pub fn find_many(
        &self,
        entity: &Entity,
        query: Query,
        find_many_options: Option<FindManyOptions>,
    ) -> DbResult<Vec<Value>> {
        let FindManyOptions { skip, limit, order } = find_many_options.unwrap_or(FindManyOptions {
            skip: None,
            limit: None,
            order: None,
        });

        // The query might have an associated query - which means we can search by the property of
        // the joined value. But the associations don't get added until a few lines down.
        let matches = self.search_with_indexes(entity, &query)?;

        let mut results: Vec<Value> = matches.into_iter().cloned().collect();

        // Problem - Associations don't get added until here.
        self.apply_associations_to_vec(&mut results, &query, entity);
        self.apply_ordering(&mut results, order);
        let paginated = self.apply_skip_limit(results, skip, limit);

        Ok(paginated)
    }

    fn apply_ordering(&self, data: &mut Vec<Value>, order: Option<Vec<FindManyOrder>>) {
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
    }

    fn apply_skip_limit(
        &self,
        data: Vec<Value>,
        skip: Option<i32>,
        limit: Option<i32>,
    ) -> Vec<Value> {
        data.into_iter()
            .skip(skip.unwrap_or(0) as usize)
            .take(limit.unwrap_or(i32::MAX) as usize)
            .collect()
    }

    pub fn apply_associations_to_vec(
        &self,
        values: &mut Vec<Value>,
        query: &Query,
        entity: &Entity,
    ) {
        for value in values.iter_mut() {
            self.apply_associations(value, query, entity);
        }
    }

    pub fn apply_associations(&self, value: &mut Value, query: &Query, entity: &Entity) {
        println!("APPLY ASSOCIATIONS");
        println!("QUERY: {query:?}");
        let associated_entities = query.associated_entities();
        println!("ASS ENT: {associated_entities:?}");
        for associated_entity in associated_entities.iter() {
            println!("FOUND ASS");
            if let Some(association) = entity
                .associations
                .iter()
                .find(|a| a.entity_name == associated_entity.name)
            {
                println!("ONE");
                if let Some(from_val) = value.get(&association.from) {
                    println!("TWO");
                    let assoc_query = Query::eq(Key(association.to.clone()), from_val.clone());
                    println!("ASS QUERY: {assoc_query:?}");
                    if let Ok(associated_data) =
                        self.find_many(associated_entity, assoc_query, None)
                    {
                        value
                            .as_object_mut()
                            .unwrap()
                            .insert(association.alias.to_string(), Value::Array(associated_data));
                    }
                }
            }
        }
    }

    // fn apply_skip_limit_order(
    //     &self,
    //     db: &Database,
    //     entity: &Entity,
    //     query: &Query,
    //     mut data: Vec<Value>,
    //     skip: Option<i32>,
    //     limit: Option<i32>,
    //     order: Option<Vec<FindManyOrder>>,
    // ) -> Vec<Value> {
    //     // Apply associations
    //     let associated_entities = query.associated_entities();
    //     for value in data.iter_mut() {
    //         for associated_entity in associated_entities.iter() {
    //             println!("ASSOCIAED ENTITIES FOUND: {associated_entity:?}");
    //             let association = entity
    //                 .associations
    //                 .iter()
    //                 .find(|a| a.entity_name == associated_entity.name);
    //             if let Some(association) = association {
    //                 if let Some(from_val) = value.get(&association.from) {
    //                     let assoc_query = Query::eq(Key(association.to.clone()), from_val.clone());
    //                     if let Ok(associated_data) =
    //                         db.find_many(associated_entity, assoc_query, None)
    //                     {
    //                         value.as_object_mut().unwrap().insert(
    //                             association.alias.to_string(),
    //                             Value::Array(associated_data),
    //                         );
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     // Order
    //     if let Some(ordering) = order {
    //         for FindManyOrder {
    //             property,
    //             direction,
    //         } in ordering.iter().rev()
    //         {
    //             data.sort_by(|a, b| {
    //                 let a_val = a.get(property).cloned().unwrap_or(Value::Null);
    //                 let b_val = b.get(property).cloned().unwrap_or(Value::Null);
    //                 let ord = compare_values(&a_val, &b_val);
    //                 match direction {
    //                     OrderDirection::Ascending => ord,
    //                     OrderDirection::Descending => ord.reverse(),
    //                 }
    //             });
    //         }
    //     }

    //     // Filter (for extra non-indexed constraints)
    //     let result = data
    //         .into_iter()
    //         .filter(|val| query.matches(val).unwrap_or(false))
    //         .skip(skip.unwrap_or(0) as usize)
    //         .take(limit.unwrap_or(i32::MAX) as usize)
    //         .collect();

    //     result
    // }

    pub fn delete_one(&mut self, entity: &Entity, query: Query) -> DbResult<Value> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;

        let data = instance
            .data
            .get_mut(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;

        // Find the key for the matching value
        let matching_key = data
            .iter()
            .find(|(_, value)| query.clone().matches(value).unwrap_or(false))
            .map(|(key, _)| key.clone())
            .ok_or_else(|| Error::msg("Value not found"))?;

        // Remove by key
        let removed = data
            .remove(&matching_key)
            .ok_or_else(|| Error::msg("Failed to remove value"))?;

        Ok(removed)
    }

    pub fn delete_many(&mut self, entity: &Entity, query: Query) -> DbResult<Vec<Value>> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found"))?;

        let data = instance
            .data
            .get_mut(&entity.name)
            .ok_or_else(|| Error::msg("Data not found"))?;

        // Collect matching keys
        let matching_keys: Vec<_> = data
            .iter()
            .filter(|(_, value)| query.clone().matches(value).unwrap_or(false))
            .map(|(key, _)| key.clone())
            .collect();

        // Remove and collect values
        let mut removed_values = Vec::new();
        for key in matching_keys {
            if let Some(val) = data.remove(&key) {
                removed_values.push(val);
            }
        }

        Ok(removed_values)
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

        // Find the matching key in the hashmap
        let matching_key = data
            .iter()
            .find_map(|(key, value)| {
                if query.clone().matches(value).unwrap_or(false) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::msg("Value not found"))?;

        let value = data
            .get_mut(&matching_key)
            .ok_or_else(|| Error::msg("Value not found"))?;

        // Merge the existing value with the update
        let new_value = match value {
            Value::Object(existing_obj) => {
                let update_obj = match update_value {
                    Value::Object(update_obj) => update_obj,
                    _ => return Err(Error::msg("Update value must be a JSON object")),
                };

                let mut merged = existing_obj.clone();
                for (k, v) in update_obj {
                    if !v.is_null() {
                        merged.insert(k, v);
                    }
                }

                Value::Object(merged)
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

        let mut updated_values = vec![];

        for (_key, value) in data.iter_mut() {
            if query.clone().matches(value).unwrap_or(false) {
                let updated_value = match value {
                    Value::Object(obj) => {
                        let update_obj = match update_value.clone() {
                            Value::Object(u) => u,
                            _ => return Err(Error::msg("Update value must be a JSON object")),
                        };

                        for (k, v) in update_obj.into_iter() {
                            if !v.is_null() {
                                obj.insert(k, v);
                            }
                        }

                        Value::Object(obj.clone()) // clone to push to return vec
                    }
                    _ => return Err(Error::msg("Stored value must be a JSON object")),
                };

                // Mutate the value in-place
                *value = updated_value.clone();
                updated_values.push(updated_value);
            }
        }

        Ok(updated_values)
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
        for value in data.values_mut() {
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
        for current in data.values_mut() {
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
