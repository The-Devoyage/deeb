use std::collections::HashMap;
use std::fs;
use std::io;

use deeb::Entity;
use deeb::InstanceName;
use serde::ser::Error;

use crate::broker::Broker;
use crate::{
    environment::Environment,
    rules::{Rules, load_rules::load_rules},
};

use super::database::Database;

#[derive(Clone)]
pub struct AppData {
    pub database: Database,
    pub environment: Environment,
    pub rules_worker: Rules,
    pub instance_name: String,
    pub broker: Broker,
}

struct SchemaInstances {
    pub instances: HashMap<InstanceName, Vec<Entity>>,
}

impl SchemaInstances {
    pub fn new(schema_json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        let instances = schema_json
            .as_object()
            .ok_or_else(|| serde_json::Error::custom("Invalid schema JSON"))?
            .iter()
            .map(|(instance_name, entity_config)| {
                let entity_config = entity_config
                    .as_object()
                    .ok_or_else(|| serde_json::Error::custom("Expected entity config object."))?;
                let entities = entity_config.get("entities");
                if entities.is_none() {
                    return Err(serde_json::Error::custom("Missing entities"));
                }
                let entities = entities
                    .unwrap()
                    .as_array()
                    .ok_or_else(|| serde_json::Error::custom("Invalid entities: expected array"))?;
                let deserialized = serde_json::from_value::<Vec<Entity>>(
                    serde_json::Value::Array(entities.clone()),
                )?;
                Ok((InstanceName(instance_name.clone()), deserialized))
            })
            .collect::<Result<HashMap<InstanceName, Vec<Entity>>, serde_json::Error>>()?;
        Ok(SchemaInstances { instances })
    }
}

impl AppData {
    pub async fn new(
        rules_path: Option<String>,
        instance_name: Option<String>,
        schema_path: Option<String>,
    ) -> Result<Self, std::io::Error> {
        let broker = Broker::new();
        let loaded_rules = load_rules(rules_path);
        let rules_worker = Rules::new(loaded_rules);
        let environment = Environment::new()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to load .env, please ensure your `.env` file is populated and placed in the same directory.: {}", e)))?;
        let database = Database::new();
        let instance_name = instance_name.unwrap_or(ulid::Ulid::new().to_string());
        let schema_path = schema_path.unwrap_or("instances.json".to_string());
        let schema = fs::read_to_string(schema_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read schema file: {}", e),
            )
        })?;
        let schema_json = serde_json::from_str::<serde_json::Value>(&schema).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to parse schema JSON: {}", e),
            )
        })?;

        let schema_instances = SchemaInstances::new(&schema_json).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to parse schema instances: {}", e),
            )
        })?;

        for instance in schema_instances.instances {
            println!("Instance: {:?}", instance);
            database
                .deeb
                .add_instance(
                    instance.0.to_string().as_str(),
                    &format!("./db/{}.json", instance_name),
                    instance.1.clone(),
                )
                .await
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to add instance: {}", e),
                    )
                })?;
        }

        Ok(AppData {
            broker,
            environment,
            database,
            rules_worker,
            instance_name,
        })
    }
}
