//! # Deeb
//! Prounced `D-B`, Deeb is an Acid Compliant JSON based database for small
//! websites and fast prototyping.
//! Inspired by simplicity of Mongo and light weight of SqLite, Deeb is a tool
//! that turns a set of JSON files into a database.

//! While performing migrations will be possible, Deeb's JSON database interface
//! allows you to simply open a json file and edit as needed.
//!
//! ## Quick Start
//!
//! ```rust
//! use deeb::*;
//! use serde_json::json;
//! use anyhow::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // Set up a new Deeb instance
//!    let db = Deeb::new();
//!    db.add_instance("test", "./user.json", vec!["user".into()])
//!     .await?;
//!    db.add_instance("test2", "./comment.json", vec!["comment".into()])
//!     .await?;
//!
//!    // Create a new entity
//!    let user = Entity::from("user");
//!    let comment = Entity::from("comment");
//!
//!    // Single Operations
//!    db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
//!    db.find_one(&user, Query::Eq("name".into(), json!("Joey")), None).await?;
//!
//!    // Perform a transaction
//!    let mut transaction = db.begin_transaction().await;
//!
//!    // Insert data into the database
//!    db.insert(&user, json!({"id": 1, "name": "Steve", "age": 3}), Some(&mut transaction)).await?;
//!    db.insert(&user, json!({"id": 2, "name": "Johnny", "age": 3}), Some(&mut transaction)).await?;
//!
//!    db.insert(&comment, json!({"user_id": 1, "comment": "Hello"}), Some(&mut transaction)).await?;
//!    db.insert(&comment, json!({"user_id": 1, "comment": "Hi"}), Some(&mut transaction)).await?;
//!
//!    // Query the database
//!    let query = Query::Eq("name".into(), json!("Steve"));
//!    let result = db.find_one(&user, query, Some(&mut transaction)).await?;
//!
//!    // Update the database
//!    let query = Query::Eq("name".into(), json!("Steve"));
//!    let update = json!({"name": "Steve", "age": 3});
//!    db.update_one(&user, query, update, Some(&mut transaction)).await?;
//!
//!    // Delete from the database
//!    let query = Query::Eq("name".into(), json!("Johnny"));
//!    db.delete_one(&user, query, Some(&mut transaction)).await?;
//!
//!    db.commit(&mut transaction).await?;
//!
//!    Ok(())
//! }
//! ```
//!

mod database;
mod deeb;

pub use crate::{
    database::{entity::Entity, query::Query},
    deeb::Deeb,
};
