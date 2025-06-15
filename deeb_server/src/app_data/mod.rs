use std::io;

use crate::environment::Environment;

use super::database::Database;

#[derive(Clone)]
pub struct AppData {
    pub database: Database,
    pub environment: Environment,
}

impl AppData {
    pub fn new() -> Result<Self, std::io::Error> {
        let environment = Environment::new()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("env load error: {}", e)))?;
        let database = Database::new();

        Ok(AppData {
            environment,
            database,
        })
    }
}
