use anyhow::Error;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    entity::Entity, name::Name, query::Query, transaction::Transaction, Database, Operation,
};

pub struct Deeb {
    db: Arc<RwLock<Database>>,
}

impl Deeb {
    pub fn new() -> Self {
        let database = Database::new();
        Self {
            db: Arc::new(RwLock::new(database)),
        }
    }

    pub async fn add_instance(
        &self,
        name: &str,
        file_path: &str,
        entities: Vec<Entity>,
    ) -> Result<&Self, Error> {
        let name = Name::from(name);
        let mut db = self.db.write().await;
        db.add_instance(name, file_path, entities);
        db.load().await?;
        Ok(self)
    }

    #[allow(dead_code)]
    pub async fn insert(
        &self,
        entity: &Entity,
        value: Value,
        transaction: Option<&mut Transaction>,
    ) -> Result<Value, Error> {
        if let Some(transaction) = transaction {
            let operation = Operation::Insert {
                entity: entity.clone(),
                value: value.clone(),
            };
            transaction.add_operation(operation);
            return Ok(value);
        }

        let mut db = self.db.write().await;
        let value = db.insert(entity, value).await?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name]).await?;
        Ok(value)
    }

    #[allow(dead_code)]
    pub async fn find_one(
        &self,
        entity: &Entity,
        query: Query,
        transaction: Option<&mut Transaction>,
    ) -> Result<Value, Error> {
        if let Some(transaction) = transaction {
            let operation = Operation::FindOne {
                entity: entity.clone(),
                query: query.clone(),
            };
            transaction.add_operation(operation);
            return Ok(Value::Null);
        }

        let db = self.db.read().await;
        let value = db.find_one(entity, query).await?;
        Ok(value)
    }

    #[allow(dead_code)]
    pub async fn find_many(
        &self,
        entity: &Entity,
        query: Query,
        transaction: Option<&mut Transaction>,
    ) -> Result<Vec<Value>, Error> {
        if let Some(transaction) = transaction {
            let operation = Operation::FindOne {
                entity: entity.clone(),
                query: query.clone(),
            };
            transaction.add_operation(operation);
            return Ok(vec![]);
        }

        let db = self.db.read().await;
        let values = db.find_many(entity, query).await?;
        Ok(values)
    }

    // Handle Transaction
    #[allow(dead_code)]
    pub async fn begin_transaction(&self) -> Transaction {
        Transaction::new()
    }

    #[allow(dead_code)]
    pub async fn commit(&self, transaction: &mut Transaction) -> Result<(), Error> {
        let mut db = self.db.write().await;
        let mut names = vec![];
        for operation in transaction.operations.iter() {
            match operation {
                Operation::Insert { entity, value } => {
                    println!("Inserting: {:?}", value);
                    db.insert(&entity, value.clone()).await?;
                    let name = db.get_instance_name_by_entity(&entity)?;
                    names.push(name.clone());
                }
                Operation::FindOne { entity, query } => {
                    println!("Finding: {:?}", query);
                    db.find_one(&entity, query.clone()).await?;
                    let name = db.get_instance_name_by_entity(&entity)?;
                    names.push(name);
                }
            }
        }
        db.commit(names).await?;
        Ok(())
    }
}
