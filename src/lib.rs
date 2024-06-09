//! # Deeb
//! websites and fast prototyping.
//! Inspired by flexibility of Mongo and light weight of SqLite, Deeb is a tool
//! that turns a set of JSON files into a database.

//! While performing migrations will be possible, Deeb's JSON database interface
//! allows you to simply open a json file and edit as needed.
//!
//! ## Quick Start
//!
//! 1. Add Deeb to your `Cargo.toml` file
//! ```bash
//! cargo add deeb
//! ```
//!
//! 2. Create a JSON file for your database.
//!
//! ```bash
//! echo '{"user": []}' > user.json
//! echo '{"comment": []}' > comment.json
//! ```
//!
//! 3. Create a deed instance and perform operations
//!
//! ```rust
//! use deeb::*;
//! use serde_json::json;
//! use anyhow::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!    // Create a new entity
//!    let user = Entity::from("user");
//!    let comment = Entity::from("comment");
//!
//!     // Set up a new Deeb instance
//!    let db = Deeb::new();
//!    db.add_instance("test", "./user.json", vec![user.clone()])
//!     .await?;
//!    db.add_instance("test2", "./comment.json", vec![comment.clone()])
//!     .await?;
//!
//!    // Single Operations
//!    db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
//!    db.find_one(&user, Query::eq("name", "Joey"), None).await?;
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
//!    let query = Query::eq("name", "Steve");
//!    let result = db.find_one(&user, query, Some(&mut transaction)).await?;
//!
//!    // Update the database
//!    let query = Query::eq("name", "Steve");
//!    let update = json!({"name": "Steve", "age": 3});
//!    db.update_one(&user, query, update, Some(&mut transaction)).await?;
//!
//!    // Delete from the database
//!    let query = Query::eq("name", "Johnny");
//!    db.delete_one(&user, query, Some(&mut transaction)).await?;
//!
//!    db.commit(&mut transaction).await?;
//!
//!    Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **ACID Compliant**: Deeb is an ACID compliant database
//! - **JSON Based**: Deeb uses JSON files as the database
//! - **Schemaless**: Deeb is schemaless
//! - **Transactions**: Deeb supports transactions
//! - **Querying**: Deeb supports querying, nested queries, and combination queries.
//!
//! ## Roadmap
//!
//! - [x] Basic CRUD Operations
//! - [x] Transactions
//! - [ ] Indexing
//! - [x] Querying
//! - [ ] Migrations
//! - [x] Benchmarks
//! - [ ] Associations
//! - [x] Documentation
//! - [x] Tests
//! - [ ] Examples
//! - [ ] Logging
//! - [ ] Error Handling
//! - [ ] CI/CD
//!
//! ## Deeb
//!
//! ### Operations
//!
//! - `insert`: [Insert](deeb::Deeb::insert) a new document into the database
//! - `find_one`: [Find](deeb::Deeb::find_one) a single document in the database
//! - `find_many`: [Find multiple](deeb::Deeb::find_many) documents in the database
//! - `update_one`: [Update a single](deeb::Deeb::update_one) document in the database
//! - `update_many`: [Update multiple](deeb::Deeb::update_many) documents in the database
//! - `delete_one`: [Delete a single](deeb::Deeb::delete_one) document in the database
//! - `delete_many`: [Delete multiple](deeb::Deeb::delete_many) documents in the database
//!
//! ### Queries
//!
//! - `eq`: [Equal](database::query::Query::eq) - Find documents based on exact match.
//! - `like`: [Like](database::query::Query::like) - Find documents based on like match.
//! - `ne`: [Not Equal](database::query::Query::ne) - Find documents based on not equal match.
//! - `gt`: [Greater Than](database::query::Query::gt) - Find documents based on greater than match.
//! - `lt`: [Less Than](database::query::Query::lt) - Find documents based on less than match.
//! - `gte`: [Greater Than or Equal](database::query::Query::gte) - Find documents based on greater than or equal match.
//! - `lte`: [Less Than or Equal](database::query::Query::lte) - Find documents based on less than or equal match.
//! - `and`: [And](database::query::Query::and) - Find documents based on multiple conditions.
//! - `or`: [Or](database::query::Query::or) - Find documents based on multiple conditions.
//!
//! ### Transactions
//!
//! - `begin_transaction`: [Begin](deeb::Deeb::begin_transaction) a new transaction
//! - `commit`: [Commit](deeb::Deeb::commit) a transaction
//!
//! ### Data Management
//!
//! - `add_key` : [Add a new key](deeb::Deeb::add_key) to the database
//! - `drop_key` : [Drop a key](deeb::Deeb::drop_key) from the database

mod database;
mod deeb;

pub use crate::{
    database::{entity::Entity, query::Query},
    deeb::Deeb,
};
