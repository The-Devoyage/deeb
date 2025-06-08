use std::collections::HashMap;

use crate::entity::{Entity, EntityName};
use serde_json::Value;

/// A database instance. Typically, a database instance is a JSON file on disk.
/// The `entities` field is a list of entities that are stored in the database used
/// by Deeb to index the data.
#[derive(Debug, Clone)]
pub struct DatabaseInstance {
    pub file_path: String,
    pub entities: Vec<Entity>,
    pub data: HashMap<EntityName, Vec<Value>>,
}
