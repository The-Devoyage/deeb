use std::io;

use crate::{environment::Environment, rules::load_rules::load_rules};

use super::database::Database;

#[derive(Clone)]
pub struct AppData {
    pub database: Database,
    pub environment: Environment,
    pub rules: String,
}

impl AppData {
    pub fn new(rules_path: Option<String>) -> Result<Self, std::io::Error> {
        let environment = Environment::new()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("env load error: {}", e)))?;
        let database = Database::new();
        let rules = load_rules(rules_path);

        Ok(AppData {
            environment,
            database,
            rules,
        })
    }
}
