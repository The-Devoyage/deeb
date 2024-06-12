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
    /// Create a new Deeb instance.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    ///    let db = Deeb::new();
    /// #   Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub fn new() -> Self {
        debug!("Creating new Deeb instance");
        let database = Database::new();
        Self {
            db: Arc::new(RwLock::new(database)),
        }
    }

    /// Add an instance to the database. An instance is a segment of the database. This
    /// is a JSON file that may have one or more entities. You can add multiple instances
    /// to the database allowing you to segment your data between different files.
    ///
    /// If the file does not exist, it will be created.
    ///
    /// The structure of the JSON file should be as follows:
    ///
    /// ```json
    /// {
    ///     "entity_name": [{...}, {...}],
    ///     "another_entity": [{...}, {...}]
    ///     ...
    /// }
    /// ```
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    ///   # let user = Entity::new("user");
    ///   # let comment = Entity::new("comment");
    ///   # let db = Deeb::new();
    ///   db.add_instance("test", "./user.json", vec![user.clone()])
    ///   .await?;
    ///   db.add_instance("test2", "./comment.json", vec![comment.clone()])
    ///   .await?;
    ///   # Ok(())
    ///   # }
    ///
    /// ```
    #[allow(dead_code)]
    pub async fn add_instance<N>(
        &self,
        name: N,
        file_path: &str,
        entities: Vec<Entity>,
    ) -> Result<&Self, Error>
    where
        N: Into<Name> + Copy,
    {
        debug!("Adding instance");
        let mut db = self.db.write().await;
        db.add_instance(&name.into(), file_path, entities);
        db.load_instance(&name.into())?;
        Ok(self)
    }

    /// Insert a single value into the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
    /// # Ok(())
    /// # }
    /// ```
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
        let value = db.insert(entity, value)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        Ok(value)
    }

    /// Insert multiple values into the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// db.insert_many(&user, vec![json!({"id": 1, "name": "Joey", "age": 10}), json!({"id": 2, "name": "Steve", "age": 3})], None).await?;
    /// # Ok(())
    /// # }
    /// ```
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
        let values = db.insert_many(entity, values)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        Ok(values)
    }

    /// Find a single value in the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
    /// db.find_one(&user, Query::eq("name", "Joey"), None).await?;
    /// # Ok(())
    /// # }
    /// ```
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
        let value = db.find_one(entity, query)?;
        trace!("Found value: {:?}", value);
        Ok(value)
    }

    /// Find multiple values in the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
    /// db.find_many(&user, Query::eq("age", 10), None).await?;
    /// # Ok(())
    /// # }
    /// ```
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
        let values = db.find_many(entity, query)?;
        trace!("Found values: {:?}", values);
        Ok(values)
    }

    /// Delete a single value from the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
    /// db.delete_one(&user, Query::eq("name", "Joey"), None).await?;
    /// # Ok(())
    /// # }
    /// ```
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
        let value = db.delete_one(entity, query)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Deleted value: {:?}", value);
        Ok(value)
    }

    /// Delete multiple values from the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// db.delete_many(&user, Query::eq("age", 10), None).await?;
    /// # Ok(())
    /// # }
    /// ```
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
        let values = db.delete_many(entity, query)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Deleted values: {:?}", values);
        Ok(values)
    }

    /// Update a single value in the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
    /// db.update_one(&user, Query::eq("age", 10), json!({"age": 3}), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn update_one(
        &self,
        entity: &Entity,
        query: Query,
        update_value: Value,
        transaction: Option<&mut Transaction>,
    ) -> Result<Value, Error> {
        debug!("Updating one");
        if let Some(transaction) = transaction {
            let operation = Operation::UpdateOne {
                entity: entity.clone(),
                query: query.clone(),
                value: update_value.clone(),
            };
            transaction.add_operation(operation);
            return Ok(update_value);
        }

        let mut db = self.db.write().await;
        let value = db.update_one(entity, query, update_value)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Updated value: {:?}", value);
        Ok(value)
    }

    /// Update multiple values in the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # db.update_many(&user, Query::eq("age", 10), json!({"age": 3}), None).await?;
    /// db.update_many(&user, Query::eq("age", 10), json!({"age": 3}), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn update_many(
        &self,
        entity: &Entity,
        query: Query,
        update_value: Value,
        transaction: Option<&mut Transaction>,
    ) -> Result<Vec<Value>, Error> {
        debug!("Updating many");
        if let Some(transaction) = transaction {
            let operation = Operation::UpdateMany {
                entity: entity.clone(),
                query: query.clone(),
                value: update_value.clone(),
            };
            transaction.add_operation(operation);
            return Ok(vec![]);
        }

        let mut db = self.db.write().await;
        let values = db.update_many(entity, query, update_value)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Updated values: {:?}", values);
        Ok(values)
    }

    // Handle Transaction
    /// Begin a new transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let db = Deeb::new();
    /// let mut transaction = db.begin_transaction().await;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn begin_transaction(&self) -> Transaction {
        debug!("Beginning transaction");
        Transaction::new()
    }

    /// Commit a transaction. Once a transaction is committed, all operations will be executed and
    /// the JSON file will be updated.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// let mut transaction = db.begin_transaction().await;
    /// db.insert(&user, json!({"id": 1, "name": "Steve", "age": 3}), Some(&mut transaction)).await?;
    /// db.insert(&user, json!({"id": 2, "name": "Johnny", "age": 3}), Some(&mut transaction)).await?;
    /// db.commit(&mut transaction).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn commit(&self, transaction: &mut Transaction) -> Result<(), Error> {
        debug!("Committing transaction");
        let mut db = self.db.write().await;
        let mut executed = vec![];
        for operation in transaction.operations.iter() {
            let result = match operation {
                Operation::InsertOne { entity, value } => db
                    .insert(&entity, value.clone())
                    .map(|value| (operation.clone(), ExecutedValue::InsertedOne(value))),
                Operation::InsertMany { entity, values } => db
                    .insert_many(&entity, values.clone())
                    .map(|values| (operation.clone(), ExecutedValue::InsertedMany(values))),
                Operation::FindOne { entity, query } => db
                    .find_one(&entity, query.clone())
                    .map(|_value| (operation.clone(), ExecutedValue::FoundOne)),
                Operation::FindMany { entity, query } => db
                    .find_many(&entity, query.clone())
                    .map(|_values| (operation.clone(), ExecutedValue::FoundMany)),
                Operation::DeleteOne { entity, query } => db
                    .delete_one(&entity, query.clone())
                    .map(|value| (operation.clone(), ExecutedValue::DeletedOne(value))),
                Operation::DeleteMany { entity, query } => db
                    .delete_many(&entity, query.clone())
                    .map(|values| (operation.clone(), ExecutedValue::DeletedMany(values))),
                Operation::UpdateOne {
                    entity,
                    query,
                    value,
                } => db
                    .update_one(&entity, query.clone(), value.clone())
                    .map(|value| (operation.clone(), ExecutedValue::UpdatedOne(value))),
                Operation::UpdateMany {
                    entity,
                    query,
                    value,
                } => db
                    .update_many(&entity, query.clone(), value.clone())
                    .map(|values| (operation.clone(), ExecutedValue::UpdatedMany(values))),
                Operation::DropKey { entity, key } => db
                    .drop_key(&entity, &key)
                    .map(|_value| (operation.clone(), ExecutedValue::DroppedKey)),
                Operation::AddKey { entity, key, value } => db
                    .add_key(&entity, &key, value.clone())
                    .map(|_value| (operation.clone(), ExecutedValue::AddedKey)),
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
                        db.delete_one(&entity, query)?;
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
                            db.delete_one(&entity, query)?;
                        }
                    }
                    _ => {}
                },
                Operation::DeleteOne { entity, .. } => match executed_value {
                    ExecutedValue::DeletedOne(value) => {
                        db.insert(&entity, value.clone()).unwrap();
                    }
                    _ => {}
                },
                Operation::DeleteMany { entity, .. } => match executed_value {
                    ExecutedValue::DeletedMany(values) => {
                        for value in values.iter() {
                            db.insert(&entity, value.clone()).unwrap();
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

    // Management

    /// Delete Key
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
    /// db.drop_key(&user, "age").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn drop_key(
        &self,
        entity: &Entity,
        key: &str,
        // transaction: Option<&mut Transaction>,
    ) -> Result<(), Error> {
        debug!("Deleting key");
        // if let Some(transaction) = transaction {
        //     let operation = Operation::DropKey {
        //         entity: entity.clone(),
        //         key: key.to_string(),
        //     };
        //     transaction.add_operation(operation);
        //     return Ok(());
        // }

        let mut db = self.db.write().await;
        db.drop_key(entity, key)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        Ok(())
    }

    /// Add key to every entity in the database.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// db.add_key(&user, "age", 10).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn add_key<V>(
        &self,
        entity: &Entity,
        key: &str,
        value: V,
        // transaction: Option<&mut Transaction>,
    ) -> Result<(), Error>
    where
        V: Into<Value> + Clone,
    {
        debug!("Adding key");
        // if let Some(transaction) = transaction {
        //     let operation = Operation::AddKey {
        //         entity: entity.clone(),
        //         key: key.to_string(),
        //         value: value.clone().into(),
        //     };
        //     transaction.add_operation(operation);
        //     return Ok(());
        // }
        let mut db = self.db.write().await;
        db.add_key(entity, key, value.into())?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        Ok(())
    }

    pub fn get_meta(&self) -> Result<Entity, Error> {
        let meta_entity = Entity::new("_meta");
        Ok(meta_entity)
    }
}
