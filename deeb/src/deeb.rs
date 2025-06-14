use anyhow::Error;
use deeb_core::database::find_many_options::FindManyOptions;
use log::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Database, Entity, ExecutedValue, InstanceName, Operation, Query, Transaction};

#[derive(Clone, Debug)]
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
        N: Into<InstanceName> + Copy,
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
    /// # use serde::{Serialize, Deserialize};
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// db.insert_one::<User>(&user, User {id: 1, name: "Joey".to_string(), age: 10}, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn insert_one<T>(
        &self,
        entity: &Entity,
        value: T,
        transaction: Option<&mut Transaction>,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned + Serialize,
    {
        debug!("Inserting");
        let value = serde_json::to_value(value)?;
        if let Some(transaction) = transaction {
            let operation = Operation::InsertOne {
                entity: entity.clone(),
                value: value.clone(),
            };
            transaction.add_operation(operation);
            return Ok(serde_json::from_value(value)?);
        }

        let mut db = self.db.write().await;
        let value = db.insert_one(entity, value)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        let typed: Result<T, _> = serde_json::from_value(value);
        Ok(typed?)
    }

    /// Insert multiple values into the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # use serde::{Serialize, Deserialize};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// db.insert_many::<User>(&user, vec![User {id: 1, name: "Joey".to_string(), age: 10}, User {id: 2, name: "Steve".to_string(), age: 3}], None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn insert_many<T>(
        &self,
        entity: &Entity,
        values: Vec<T>,
        transaction: Option<&mut Transaction>,
    ) -> Result<Vec<T>, Error>
    where
        T: Serialize + DeserializeOwned,
    {
        debug!("Inserting many");
        let values: Vec<Value> = values
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?;
        if let Some(transaction) = transaction {
            let operation = Operation::InsertMany {
                entity: entity.clone(),
                values: values.clone(),
            };
            transaction.add_operation(operation);
            let typed: Result<Vec<T>, _> = values.into_iter().map(serde_json::from_value).collect();
            return Ok(typed?);
        }

        let mut db = self.db.write().await;
        let values = db.insert_many(entity, values)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        let typed: Result<Vec<T>, _> = values.into_iter().map(serde_json::from_value).collect();
        Ok(typed?)
    }

    /// Find a single value in the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # use serde::{Serialize, Deserialize};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// # db.insert_one::<User>(&user, User {id: 1, name: "Joey D".to_string(), age: 10}, None).await?;
    /// db.find_one::<User>(&user, Query::eq("name", "Joey D"), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn find_one<T>(
        &self,
        entity: &Entity,
        query: Query,
        transaction: Option<&mut Transaction>,
    ) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned,
    {
        debug!("Finding one");
        if let Some(transaction) = transaction {
            let operation = Operation::FindOne {
                entity: entity.clone(),
                query: query.clone(),
            };
            transaction.add_operation(operation);
            return Ok(None);
        }
        println!("Finding one: {:?}", entity);

        let db = self.db.read().await;
        let value = db.find_one(entity, query).ok();
        trace!("Found value: {:?}", value);
        match value {
            Some(v) => Ok(Some(serde_json::from_value(v)?)),
            None => Ok(None),
        }
    }

    /// Find multiple values in the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # use serde::{Serialize, Deserialize};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// # db.insert_one::<User>(&user, User {id: 1, name: "Joey".to_string(), age: 10}, None).await?;
    /// db
    /// .find_many::<User>(
    ///     &user,
    ///     Query::eq("age", 10),
    ///     Some(FindManyOptions{
    ///         skip: None,
    ///         limit: Some(10),
    ///         order: Some(vec![
    ///             FindManyOrder {
    ///                 property: "name".to_string(),
    ///                 direction: OrderDirection::Ascending
    ///
    ///             }
    ///         ])
    ///     }),
    ///     None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn find_many<T>(
        &self,
        entity: &Entity,
        query: Query,
        find_many_options: Option<FindManyOptions>,
        transaction: Option<&mut Transaction>,
    ) -> Result<Option<Vec<T>>, Error>
    where
        T: DeserializeOwned,
    {
        debug!("Finding many");
        if let Some(transaction) = transaction {
            let operation = Operation::FindMany {
                entity: entity.clone(),
                query: query.clone(),
                find_many_options,
            };
            transaction.add_operation(operation);
            return Ok(None);
        }

        let db = self.db.read().await;
        let values = db.find_many(entity, query, find_many_options)?;
        trace!("Found values: {:?}", values);
        let typed: Result<Vec<T>, _> = values.into_iter().map(serde_json::from_value).collect();
        Ok(Some(typed?))
    }

    /// Delete a single value from the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # use serde::{Serialize, Deserialize};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// # db.insert_one::<User>(&user, User {id: 1, name: "Joey".to_string(), age: 10}, None).await?;
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
    ) -> Result<Option<bool>, Error> {
        debug!("Deleting one");
        if let Some(transaction) = transaction {
            let operation = Operation::DeleteOne {
                entity: entity.clone(),
                query: query.clone(),
            };
            transaction.add_operation(operation);
            return Ok(None);
        }

        let mut db = self.db.write().await;
        let value = db.delete_one(entity, query)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Deleted value: {:?}", value);
        Ok(Some(true))
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
    ) -> Result<Option<bool>, Error> {
        debug!("Deleting many");
        if let Some(transaction) = transaction {
            let operation = Operation::DeleteMany {
                entity: entity.clone(),
                query: query.clone(),
            };
            transaction.add_operation(operation);
            return Ok(None);
        }

        let mut db = self.db.write().await;
        let values = db.delete_many(entity, query)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Deleted values: {:?}", values);
        Ok(Some(true))
    }

    /// Update a single value in the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # use serde::{Serialize, Deserialize};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// # #[derive(Serialize)]
    /// # struct UpdateUser {
    /// #   age: Option<i32>,
    /// #   name: Option<String>
    /// # }
    /// # db.insert_one::<User>(&user, User {id: 1, name: "Joey".to_string(), age: 10}, None).await?;
    /// db.update_one::<User, UpdateUser>(&user, Query::eq("age", 10), UpdateUser{age: Some(3), name: None}, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn update_one<T, K>(
        &self,
        entity: &Entity,
        query: Query,
        update_value: K,
        transaction: Option<&mut Transaction>,
    ) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned,
        K: Serialize,
    {
        debug!("Updating one");

        let update_value = serde_json::to_value(update_value)?;

        if let Some(transaction) = transaction {
            let operation = Operation::UpdateOne {
                entity: entity.clone(),
                query: query.clone(),
                value: update_value.clone(),
            };
            transaction.add_operation(operation);
            return Ok(None);
        }

        let mut db = self.db.write().await;
        let value = db.update_one(entity, query, update_value)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Updated value: {:?}", value);
        Ok(Some(serde_json::from_value(value)?))
    }

    /// Update multiple values in the database.
    /// Passing a transaction will queue the operation to be executed later and
    /// requires you to commit the transaction.
    ///
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # use serde::{Serialize, Deserialize};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// # #[derive(Serialize)]
    /// # struct UpdateUser {
    /// #   age: Option<i32>,
    /// #   name: Option<String>
    /// # }
    /// # db.insert_many::<User>(&user, vec![User {id: 1938, name: "Tula".to_string(), age: 7}, User {id: 13849, name: "Bulla".to_string(), age: 7}], None).await?;
    /// db.update_many::<User, UpdateUser>(&user, Query::eq("age", 7), UpdateUser {age: Some(8), name: None}, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn update_many<T, K>(
        &self,
        entity: &Entity,
        query: Query,
        update_value: K,
        transaction: Option<&mut Transaction>,
    ) -> Result<Option<Vec<T>>, Error>
    where
        T: DeserializeOwned,
        K: Serialize,
    {
        debug!("Updating many");
        let update_value = serde_json::to_value(update_value)?;
        if let Some(transaction) = transaction {
            let operation = Operation::UpdateMany {
                entity: entity.clone(),
                query: query.clone(),
                value: update_value.clone(),
            };
            transaction.add_operation(operation);
            return Ok(None);
        }

        let mut db = self.db.write().await;
        let values = db.update_many(entity, query, update_value)?;
        let name = db.get_instance_name_by_entity(entity)?;
        db.commit(vec![name])?;
        trace!("Updated values: {:?}", values);
        let typed: Result<Vec<T>, _> = values.into_iter().map(serde_json::from_value).collect();
        Ok(Some(typed?))
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
    /// # use serde::{Serialize, Deserialize};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// let mut transaction = db.begin_transaction().await;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// db.insert_one::<User>(&user, User {id: 1, name: "Steve".to_string(), age: 3}, Some(&mut transaction)).await?;
    /// db.insert_one::<User>(&user, User {id: 2, name: "Johnny".to_string(), age: 3}, Some(&mut transaction)).await?;
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
                    .insert_one(&entity, value.clone())
                    .map(|value| (operation.clone(), ExecutedValue::InsertedOne(value))),
                Operation::InsertMany { entity, values } => db
                    .insert_many(&entity, values.clone())
                    .map(|values| (operation.clone(), ExecutedValue::InsertedMany(values))),
                Operation::FindOne { entity, query } => db
                    .find_one(&entity, query.clone())
                    .map(|_value| (operation.clone(), ExecutedValue::FoundOne)),
                Operation::FindMany {
                    entity,
                    query,
                    find_many_options,
                } => db
                    .find_many(&entity, query.clone(), find_many_options.clone())
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
                        db.insert_one(&entity, value.clone()).unwrap();
                    }
                    _ => {}
                },
                Operation::DeleteMany { entity, .. } => match executed_value {
                    ExecutedValue::DeletedMany(values) => {
                        for value in values.iter() {
                            db.insert_one(&entity, value.clone()).unwrap();
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
    /// A utility method to remove a key from every document in the collection.
    /// ```
    /// # use deeb::*;
    /// # use anyhow::Error;
    /// # use serde_json::json;
    /// # use serde::{Serialize, Deserialize};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let user = Entity::new("user");
    /// # let db = Deeb::new();
    /// # db.add_instance("test", "./user.json", vec![user.clone()]).await?;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #   id: i32,
    /// #   name: String,
    /// #   age: i32
    /// # }
    /// # db.insert_one::<User>(&user, User {id: 1, name: "Joey".to_string(), age: 10}, None).await?;
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

    /// Construct the Meta entity
    pub fn get_meta(&self) -> Result<Entity, Error> {
        let meta_entity = Entity::new("_meta");
        Ok(meta_entity)
    }
}
