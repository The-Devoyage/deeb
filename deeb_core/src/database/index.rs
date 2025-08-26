use anyhow::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::entity::Entity;

use super::{Database, DbResult, query::Query};

pub type EntityID = String;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub keys: Vec<String>,
    pub options: Option<IndexOptions>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct IndexOptions {
    pub unique: bool,
    pub sparse: bool,
    pub case_insensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexKey {
    Single(ValueKey),
    Compound(Vec<ValueKey>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueKey {
    Null,
    Bool(bool),
    Number(i64),
    String(String),
}

#[derive(Debug, Clone)]
pub struct BuiltIndex {
    pub keys: Vec<String>,
    pub map: BTreeMap<IndexKey, Vec<EntityID>>,
}

#[derive(Debug, Clone)]
pub struct IndexStore {
    pub indexes: Vec<BuiltIndex>,
}

pub fn value_to_key(value: &Value) -> Option<ValueKey> {
    match value {
        Value::String(s) => Some(ValueKey::String(s.clone())),
        Value::Number(n) => n.as_i64().map(ValueKey::Number),
        Value::Bool(b) => Some(ValueKey::Bool(*b)),
        //TODO: This should produce error
        _ => None,
    }
}

impl Database {
    /// Called after entity insertion into an instance.
    /// Selects every document and indexes by the entities indexes.
    pub fn build_index(&mut self, entity: &Entity) -> DbResult<()> {
        let mut built_indexes = Vec::<BuiltIndex>::new();
        log::debug!("BUILD INDEX");
        let documents = self.find_many(entity, Query::All, None).unwrap_or(vec![]);

        // Get the defined indexes
        for index_def in &entity.indexes {
            let keys = &index_def.keys;
            if keys.is_empty() {
                continue;
            }

            let mut map = BTreeMap::new();

            // For each document
            for document in &documents {
                let mut key_parts = Vec::new();
                let mut skip = false;

                // Create the value keys
                for col in keys {
                    match document.get(col).and_then(value_to_key) {
                        Some(part) => key_parts.push(part),
                        None => {
                            skip = true;
                            break;
                        }
                    }
                }

                if skip {
                    continue;
                }

                let key = if key_parts.len() == 1 {
                    IndexKey::Single(key_parts[0].clone())
                } else {
                    IndexKey::Compound(key_parts)
                };

                if let Some(_id) = document.get("_id").and_then(|v| v.as_str()) {
                    map.entry(key)
                        .or_insert_with(Vec::new)
                        .push(_id.to_string());
                }
            }

            built_indexes.push(BuiltIndex {
                keys: keys.to_vec(),
                map,
            });
        }

        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or(Error::msg("Failed to find instance while indexing."))?;

        let index_store = IndexStore {
            indexes: built_indexes,
        };

        instance.indexes.insert(entity.name.clone(), index_store);

        Ok(())
    }

    pub fn append_indexes(&mut self, entity: &Entity, inserted: &[Value]) -> DbResult<()> {
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found for indexing"))?;

        let index_store = instance
            .indexes
            .entry(entity.name.clone())
            .or_insert_with(|| IndexStore { indexes: vec![] });

        for index_def in &entity.indexes {
            let keys = &index_def.keys;
            if keys.is_empty() {
                continue;
            }

            // Find matching built index or create new one
            let built_index = index_store.indexes.iter_mut().find(|idx| idx.keys == *keys);

            let index_map = if let Some(existing) = built_index {
                &mut existing.map
            } else {
                index_store.indexes.push(BuiltIndex {
                    keys: keys.clone(),
                    map: BTreeMap::new(),
                });
                &mut index_store.indexes.last_mut().unwrap().map
            };

            for document in inserted {
                let mut key_parts = Vec::new();
                let mut skip = false;

                for col in keys {
                    match document.get(col).and_then(value_to_key) {
                        Some(part) => key_parts.push(part),
                        None => {
                            skip = true;
                            break;
                        }
                    }
                }

                if skip {
                    continue;
                }

                let key = if key_parts.len() == 1 {
                    IndexKey::Single(key_parts[0].clone())
                } else {
                    IndexKey::Compound(key_parts)
                };

                if let Some(_id) = document.get("_id").and_then(|v| v.as_str()) {
                    index_map
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(_id.to_string());
                }
            }
        }

        Ok(())
    }

    pub fn update_indexes(
        &mut self,
        entity: &Entity,
        old_value: &Value,
        new_value: &Value,
    ) -> DbResult<()> {
        // 1. Find the instance and index store.
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found for indexing"))?;

        let index_store = instance
            .indexes
            .entry(entity.name.clone())
            .or_insert_with(|| IndexStore { indexes: vec![] });

        // 2. For each index definition...
        for index_def in &entity.indexes {
            let keys = &index_def.keys;
            if keys.is_empty() {
                continue;
            }

            // Find the corresponding built index.
            let built_index = match index_store.indexes.iter_mut().find(|idx| &idx.keys == keys) {
                Some(idx) => idx,
                None => continue, // Or create it if it doesn't exist? For now, skip.
            };

            // 3. Remove the old entry from the index.
            let mut old_key_parts = Vec::new();
            let mut skip_old = false;
            for col in keys {
                match old_value.get(col).and_then(value_to_key) {
                    Some(part) => old_key_parts.push(part),
                    None => {
                        skip_old = true;
                        break;
                    }
                }
            }

            if !skip_old {
                let old_key = if old_key_parts.len() == 1 {
                    IndexKey::Single(old_key_parts[0].clone())
                } else {
                    IndexKey::Compound(old_key_parts)
                };
                if let Some(ids) = built_index.map.get_mut(&old_key) {
                    if let Some(pos) = ids.iter().position(|id| {
                        id == old_value
                            .get("_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                    }) {
                        ids.remove(pos);
                    }
                }
            }

            // 4. Add the new entry to the index.
            let mut new_key_parts = Vec::new();
            let mut skip_new = false;
            for col in keys {
                match new_value.get(col).and_then(value_to_key) {
                    Some(part) => new_key_parts.push(part),
                    None => {
                        skip_new = true;
                        break;
                    }
                }
            }

            if !skip_new {
                let new_key = if new_key_parts.len() == 1 {
                    IndexKey::Single(new_key_parts[0].clone())
                } else {
                    IndexKey::Compound(new_key_parts)
                };
                if let Some(_id) = new_value.get("_id").and_then(|v| v.as_str()) {
                    built_index
                        .map
                        .entry(new_key)
                        .or_insert_with(Vec::new)
                        .push(_id.to_string());
                }
            }
        }

        Ok(())
    }

    pub fn delete_indexes(&mut self, entity: &Entity, deleted: &[Value]) -> DbResult<()> {
        // 1. Find the instance and index store.
        let instance = self
            .get_instance_by_entity_mut(entity)
            .ok_or_else(|| Error::msg("Entity not found for indexing"))?;

        let index_store = match instance.indexes.get_mut(&entity.name) {
            Some(store) => store,
            None => return Ok(()), // No indexes for this entity, nothing to do.
        };

        // For each deleted document
        for document in deleted {
            // For each index definition...
            for index_def in &entity.indexes {
                let keys = &index_def.keys;
                if keys.is_empty() {
                    continue;
                }

                // Find the corresponding built index.
                let built_index = match index_store.indexes.iter_mut().find(|idx| &idx.keys == keys)
                {
                    Some(idx) => idx,
                    None => continue, // Index doesn't exist, skip.
                };

                // Create the key for the document to be deleted.
                let mut key_parts = Vec::new();
                let mut skip = false;
                for col in keys {
                    match document.get(col).and_then(value_to_key) {
                        Some(part) => key_parts.push(part),
                        None => {
                            skip = true;
                            break;
                        }
                    }
                }

                if skip {
                    continue;
                }

                let key = if key_parts.len() == 1 {
                    IndexKey::Single(key_parts[0].clone())
                } else {
                    IndexKey::Compound(key_parts)
                };

                // Remove the document's ID from the index entry.
                if let Some(ids) = built_index.map.get_mut(&key) {
                    if let Some(id_to_delete) = document.get("_id").and_then(|v| v.as_str()) {
                        if let Some(pos) = ids.iter().position(|id| id == id_to_delete) {
                            ids.remove(pos);
                        }
                        // If the vec is empty after removal, remove the key from the map.
                        if ids.is_empty() {
                            built_index.map.remove(&key);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
