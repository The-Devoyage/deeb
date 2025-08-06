use std::collections::HashMap;

use crate::entity::{Entity, EntityName, PrimaryKey};
use serde_json::Value;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

use super::DbResult;
use super::index::IndexStore;

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub enum PrimaryKeyValue {
    String(String),
    Number(i64),
}

impl PartialEq for PrimaryKeyValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PrimaryKeyValue::String(a), PrimaryKeyValue::String(b)) => a == b,
            (PrimaryKeyValue::Number(a), PrimaryKeyValue::Number(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for PrimaryKeyValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PrimaryKeyValue::String(s) => {
                state.write_u8(0);
                s.hash(state);
            }
            PrimaryKeyValue::Number(n) => {
                state.write_u8(1);
                n.hash(state);
            }
        }
    }
}

impl fmt::Display for PrimaryKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimaryKeyValue::String(s) => write!(f, "{}", s),
            PrimaryKeyValue::Number(n) => write!(f, "{}", n),
        }
    }
}

impl From<&Value> for PrimaryKeyValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(s) => PrimaryKeyValue::String(s.clone()),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    PrimaryKeyValue::String(i.to_string())
                } else {
                    panic!("Only i64 number keys are supported");
                }
            }
            _ => panic!("Unsupported primary key type"),
        }
    }
}

impl PrimaryKeyValue {
    pub fn new(value: &Value, primary_key: &PrimaryKey) -> DbResult<PrimaryKeyValue> {
        let primary_key_value = value.get(primary_key.0.clone()).ok_or_else(|| {
            log::error!("Failed to get primary key value.");
            anyhow::Error::msg(format!(
                "Failed to read primary key `{:?}` in value {:?}",
                primary_key, value,
            ))
        })?;

        Ok(PrimaryKeyValue::from(primary_key_value))
    }
}

type InstanceData = HashMap<String, Value>;

/// A database instance. Typically, a database instance is a JSON file on disk.
/// The `entities` field is a list of entities that are stored in the database used
/// by Deeb to index the data.
#[derive(Debug, Clone)]
pub struct DatabaseInstance {
    pub file_path: String,
    pub entities: Vec<Entity>,
    pub data: HashMap<EntityName, InstanceData>,
    pub indexes: HashMap<EntityName, IndexStore>,
}

impl DatabaseInstance {
    /// Fetch the data instance `instance.data.{entity_name}` or initalize with an empty hash map.
    pub fn get_or_init(&mut self, entity_name: &EntityName) -> &mut InstanceData {
        let instance_data = self
            .data
            .entry(entity_name.clone())
            .or_insert(HashMap::new());

        instance_data
    }
}
