use anyhow::Error;
use log::*;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    entity::Entity, name::Name, query::Query, transaction::Transaction, Database, ExecutedValue,
    Operation,
};

pub struct Deeb {
    db: Arc<RwLock<Database>>,
}

impl Deeb {
    #[allow(dead_code)]
    pub fn new() -> Self {
        debug!("Creating new Deeb instance");
        let database = Database::new();
        Self {
            db: Arc::new(RwLock::new(database)),
        }
    }

    #[allow(dead_code)]
    pub async fn add_instance(
        &self,
        name: &str,
        file_path: &str,
        entities: Vec<Entity>,
    ) -> Result<&Self, Error> {
        debug!("Adding instance");
        let name = Name::from(name);
        let mut db = self.db.write().await;
        db.add_instance(name.clone(), file_path, entities);
        db.load_instance(&name)?;
        Ok(self)
    }

    #[allow(dead_code)]
    pub async fn insert(
        &self,
        entity: &Entity,
        value: Value,
        transaction: Option<&mut Transaction>,
    ) -> Result<Value, Error> {
        debug!("Inserting");
        if let Some(transaction) = transaction {
            let operation = Operation::InsertOne {
                entity: entity.clone(),
                value: value.clone(),
            };
            transaction.add_operation(operation);
            return Ok(value);
        }

        let mut db = self.db.write().await;
        let value = db.insert(entity, value).await?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        Ok(value)
    }

    #[allow(dead_code)]
    pub async fn insert_many(
        &self,
        entity: &Entity,
        values: Vec<Value>,
        transaction: Option<&mut Transaction>,
    ) -> Result<Vec<Value>, Error> {
        debug!("Inserting many");
        if let Some(transaction) = transaction {
            let operation = Operation::InsertMany {
                entity: entity.clone(),
                values: values.clone(),
            };
            transaction.add_operation(operation);
            return Ok(values);
        }

        let mut db = self.db.write().await;
        let values = db.insert_many(entity, values).await?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        Ok(values)
    }

    #[allow(dead_code)]
    pub async fn find_one(
        &self,
        entity: &Entity,
        query: Query,
        transaction: Option<&mut Transaction>,
    ) -> Result<Value, Error> {
        debug!("Finding one");
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
        trace!("Found value: {:?}", value);
        Ok(value)
    }

    #[allow(dead_code)]
    pub async fn find_many(
        &self,
        entity: &Entity,
        query: Query,
        transaction: Option<&mut Transaction>,
    ) -> Result<Vec<Value>, Error> {
        debug!("Finding many");
        if let Some(transaction) = transaction {
            let operation = Operation::FindMany {
                entity: entity.clone(),
                query: query.clone(),
            };
            transaction.add_operation(operation);
            return Ok(vec![]);
        }

        let db = self.db.read().await;
        let values = db.find_many(entity, query).await?;
        trace!("Found values: {:?}", values);
        Ok(values)
    }

    #[allow(dead_code)]
    pub async fn delete_one(
        &self,
        entity: &Entity,
        query: Query,
        transaction: Option<&mut Transaction>,
    ) -> Result<Value, Error> {
        debug!("Deleting one");
        if let Some(transaction) = transaction {
            let operation = Operation::DeleteOne {
                entity: entity.clone(),
                query: query.clone(),
            };
            transaction.add_operation(operation);
            return Ok(Value::Null);
        }

        let mut db = self.db.write().await;
        let value = db.delete_one(entity, query).await?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Deleted value: {:?}", value);
        Ok(value)
    }

    #[allow(dead_code)]
    pub async fn delete_many(
        &self,
        entity: &Entity,
        query: Query,
        transaction: Option<&mut Transaction>,
    ) -> Result<Vec<Value>, Error> {
        debug!("Deleting many");
        if let Some(transaction) = transaction {
            let operation = Operation::DeleteMany {
                entity: entity.clone(),
                query: query.clone(),
            };
            transaction.add_operation(operation);
            return Ok(vec![]);
        }

        let mut db = self.db.write().await;
        let values = db.delete_many(entity, query).await?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Deleted values: {:?}", values);
        Ok(values)
    }

    // Handle Transaction
    #[allow(dead_code)]
    pub async fn begin_transaction(&self) -> Transaction {
        debug!("Beginning transaction");
        Transaction::new()
    }

    #[allow(dead_code)]
    pub async fn commit(&self, transaction: &mut Transaction) -> Result<(), Error> {
        debug!("Committing transaction");
        let mut db = self.db.write().await;
        let mut executed = vec![];
        for operation in transaction.operations.iter() {
            let result = match operation {
                Operation::InsertOne { entity, value } => db
                    .insert(&entity, value.clone())
                    .await
                    .map(|value| (operation.clone(), ExecutedValue::InsertedOne(value))),
                Operation::InsertMany { entity, values } => db
                    .insert_many(&entity, values.clone())
                    .await
                    .map(|values| (operation.clone(), ExecutedValue::InsertedMany(values))),
                Operation::FindOne { entity, query } => db
                    .find_one(&entity, query.clone())
                    .await
                    .map(|_value| (operation.clone(), ExecutedValue::FoundOne)),
                Operation::FindMany { entity, query } => db
                    .find_many(&entity, query.clone())
                    .await
                    .map(|_values| (operation.clone(), ExecutedValue::FoundMany)),
                Operation::DeleteOne { entity, query } => db
                    .delete_one(&entity, query.clone())
                    .await
                    .map(|value| (operation.clone(), ExecutedValue::DeletedOne(value))),
                Operation::DeleteMany { entity, query } => db
                    .delete_many(&entity, query.clone())
                    .await
                    .map(|values| (operation.clone(), ExecutedValue::DeletedMany(values))),
            };
            trace!("Executed operation: {:?}", operation);

            match result {
                Ok(executed_value) => executed.push(executed_value),
                Err(err) => {
                    trace!("Error occurred: {:?}", err);
                    drop(db);
                    self.rollback(&mut executed).await?;
                    return Err(err);
                }
            }
        }

        let mut names = vec![];
        for (operation, _executed_value) in executed.iter() {
            trace!("Getting names");
            let entity = match operation {
                Operation::InsertOne { entity, .. } => entity,
                Operation::DeleteOne { entity, .. } => entity,
                Operation::DeleteMany { entity, .. } => entity,
                _ => continue,
            };
            let name = db.get_instance_name_by_entity(entity).unwrap();
            names.push(name);
        }
        trace!("Names: {:?}", names);

        db.commit(names)?;
        trace!("Executed operations: {:?}", executed);
        Ok(())
    }

    async fn rollback(&self, executed: &mut Vec<(Operation, ExecutedValue)>) -> Result<(), Error> {
        debug!("Rolling back transaction");
        let mut db = self.db.write().await;
        for (operation, executed_value) in executed.iter().rev() {
            match operation {
                Operation::InsertOne { entity, .. } => match executed_value {
                    ExecutedValue::InsertedOne(value) => {
                        let query = Query::and(
                            value
                                .as_object()
                                .unwrap()
                                .iter()
                                .map(|(key, value)| {
                                    Query::Eq(key.clone().as_str().into(), value.clone())
                                })
                                .collect::<Vec<_>>(),
                        );
                        db.delete_one(&entity, query).await?;
                    }
                    _ => {}
                },
                Operation::InsertMany { entity, .. } => match executed_value {
                    ExecutedValue::InsertedMany(values) => {
                        for value in values.iter() {
                            let query = Query::and(
                                value
                                    .as_object()
                                    .unwrap()
                                    .iter()
                                    .map(|(key, value)| {
                                        Query::Eq(key.clone().as_str().into(), value.clone())
                                    })
                                    .collect::<Vec<_>>(),
                            );
                            db.delete_one(&entity, query).await?;
                        }
                    }
                    _ => {}
                },
                Operation::DeleteOne { entity, .. } => match executed_value {
                    ExecutedValue::DeletedOne(value) => {
                        db.insert(&entity, value.clone()).await.unwrap();
                    }
                    _ => {}
                },
                Operation::DeleteMany { entity, .. } => match executed_value {
                    ExecutedValue::DeletedMany(values) => {
                        for value in values.iter() {
                            db.insert(&entity, value.clone()).await.unwrap();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        trace!("Rolled back operations");
        Ok(())
    }
}
