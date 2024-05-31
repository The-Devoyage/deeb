use anyhow::Error;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{entity::Entity, name::Name, Database};

pub struct Deeb {
    db: Arc<Mutex<Database>>,
}

impl Deeb {
    pub fn new() -> Self {
        let database = Database::new();
        Self {
            db: Arc::new(Mutex::new(database)),
        }
    }

    pub async fn add_instance(
        &self,
        name: Name,
        file_path: &str,
        entities: Vec<Entity>,
    ) -> Result<&Self, Error> {
        println!("Adding instance");
        let mut db = self.db.lock().await;
        db.add_instance(name, file_path, entities);
        println!("Loading database");
        db.load().await?;
        println!("Database loaded");
        Ok(self)
    }

    #[allow(dead_code)]
    pub async fn insert(&self, entity: &Entity, insert: Value) -> Result<Value, Error> {
        println!("Inserting value");
        let mut db = self.db.lock().await;
        let value = db.insert(entity, insert).await?;
        db.commit("test".into()).await?;
        Ok(value)
    }

    #[allow(dead_code)]
    pub async fn find_one(&self, entity: &Entity, query: Value) -> Result<Value, Error> {
        let db = self.db.lock().await;
        let value = db.find_one(entity, query).await?;
        Ok(value)
    }
}
