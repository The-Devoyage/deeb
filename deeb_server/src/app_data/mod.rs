use std::io;

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
}

impl AppData {
    pub fn new(
        rules_path: Option<String>,
        instance_name: Option<String>,
    ) -> Result<Self, std::io::Error> {
        let loaded_rules = load_rules(rules_path);
        let rules_worker = Rules::new(loaded_rules);
        let environment = Environment::new()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to load .env, please ensure your `.env` file is populated and placed in the same directory.: {}", e)))?;
        let database = Database::new();
        let instance_name = instance_name.unwrap_or(ulid::Ulid::new().to_string());

        Ok(AppData {
            environment,
            database,
            rules_worker,
            instance_name,
        })
    }
}
