use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entity::Entity;

use super::{Database, DbResult, query::Query};

pub type EntityID = String;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
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
    pub column: String,
    pub map: BTreeMap<IndexKey, Vec<EntityID>>,
}

#[derive(Debug, Clone)]
pub struct IndexStore {
    pub indexes: Vec<BuiltIndex>,
}

fn value_to_key(value: &Value) -> Option<ValueKey> {
    match value {
        Value::String(s) => Some(ValueKey::String(s.clone())),
        Value::Number(n) => n.as_i64().map(ValueKey::Number),
        Value::Bool(b) => Some(ValueKey::Bool(*b)),
        //TODO: This should produce error
        _ => None,
    }
}

impl Database {
    pub fn build_index(&self, entity: &Entity) -> DbResult<IndexStore> {
        let mut built_indexes = Vec::<BuiltIndex>::new();
        let rows = self.find_many(entity, Query::All, None)?;

        for index_def in &entity.indexes {
            let columns = &index_def.columns;
            if columns.is_empty() {
                continue;
            }

            let mut map = BTreeMap::new();

            for row in &rows {
                let mut key_parts = Vec::new();
                let mut skip = false;

                for col in columns {
                    match row.get(col).and_then(value_to_key) {
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

                if let Some(_id) = row.get("_id").and_then(|v| v.as_str()) {
                    map.entry(key)
                        .or_insert_with(Vec::new)
                        .push(_id.to_string());
                }
            }

            built_indexes.push(BuiltIndex {
                // TODO: store Vec<String> instead of just one column
                column: columns.join(","),
                map,
            });
        }

        Ok(IndexStore {
            indexes: built_indexes,
        })
    }
}
