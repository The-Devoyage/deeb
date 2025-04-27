//! # Deeb - JSON Database
//!
//! Call it “Deeb,” “D-b,” or “That Cool JSON Thing”—this ACID Compliant database
//! is perfect for tiny sites and rapid experiments.
//!
//! Inspired by flexibility of Mongo and light weight of SqLite, Deeb is a tool
//! that turns a set of JSON files into a light weight database.
//!
//! Deeb's ability to turn groups JSON files into a database allows you to simply
//! open a json file and edit as needed.
//!
//! Check out the quick start below, or the [docs](https://docs.rs/deeb/latest/deeb/)
//! to learn more.
//!
//! ## Quick Start
//!
//! 1. Add Deeb to your `Cargo.toml` file
//!
//! ```bash
//! cargo add deeb
//! ```
//!
//! 2. Optionally, Create a JSON file for your database. Deeb will also create one for you if you like.
//!
//! ```bash
//! echo '{"user": []}' > user.json
//! echo '{"comment": []}' > comment.json
//! ```
//!
//! **Terminology**
//! - Instance: A single .json file managed by Deeb. Each instance can store multiple entities and serves as a lightweight, self-contained database.
//! - Collection: Similar to a table (SQL) or collection (MongoDB), an array of entity documents of a consistent type within an instance.
//! - Entity: The `type` of document within a collection, for example User or Comment.
//! - Document: An individual record representing an entity. Documents are stored as JSON objects and represent a single unit of data (e.g., a user, message, or task).
//!
//! 3. Create a deeb instance and perform operations.
//!
//! ```rust
//! use deeb::*;
//! use serde_json::json;
//! use serde::{Serialize, Deserialize};
//! use anyhow::Error;
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     id: i32,
//!     name: String,
//!     age: i32
//! }
//!
//! #[derive(Serialize)]
//! struct UpdateUser {
//!     name: Option<String>,
//!     age: Option<i32>
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct Comment {
//!     user_id: i32,
//!     comment: String
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!    // Create a new entity
//!    let user = Entity::new("user");
//!    let comment = Entity::new("comment");
//!
//!     // Set up a new Deeb instance
//!    let db = Deeb::new();
//!    db.add_instance("test", "./user.json", vec![user.clone()])
//!     .await?;
//!    db.add_instance("test2", "./comment.json", vec![comment.clone()])
//!     .await?;
//!
//!    // Single Operations
//!    db.insert::<User>(&user, User {id: 1, name: "George".to_string(), age: 10}, None).await?;
//!    db.find_one::<User>(&user, Query::eq("name", "George"), None).await?;
//!
//!    // Perform a transaction
//!    let mut transaction = db.begin_transaction().await;
//!
//!    // Insert data into the database
//!    db.insert::<User>(&user, User {id: 1, name: "Steve".to_string(), age: 3}, Some(&mut transaction)).await?;
//!    db.insert::<User>(&user, User {id: 2, name: "Johnny".to_string(), age: 3}, Some(&mut transaction)).await?;
//!
//!    db.insert::<Comment>(&comment, Comment {user_id: 1, comment: "Hello".to_string()}, Some(&mut transaction)).await?;
//!    db.insert::<Comment>(&comment, Comment {user_id: 1, comment: "Hi".to_string()}, Some(&mut transaction)).await?;
//!
//!    // Query the database
//!    let query = Query::eq("name", "Steve");
//!    let result = db.find_one::<User>(&user, query, Some(&mut transaction)).await?;
//!
//!    // Update the database
//!    let query = Query::eq("name", "Steve");
//!    let update = UpdateUser { age: Some(5), name: None };
//!    db.update_one::<User, UpdateUser>(&user, query, update, Some(&mut transaction)).await?;
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
//! - **ACID Compliant**: Deeb is an ACID compliant database. We get close as we can for a light weight JSON based DB.
//! - **JSON-Based Storage**: Deeb uses lightweight JSON files as the underlying data store, providing human-readable structure and seamless integration with any system that speaks JSON.
//! - **Schemaless**: Deeb doesn't require a predefined schema like traditional SQL or strongly-typed NoSQL databases. However, by using Rust generics, you can enforce type safety at compile time. This means Deeb stays flexible at runtime, while giving you confidence at build time.
//! - **Transactions**: Perform multiple operations as a single unit — commit them all at once or roll them back if something fails.
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
//! - [x] Associations
//! - [x] Documentation
//! - [x] Tests
//! - [x] Examples
//! - [ ] Logging
//! - [ ] Error Handling
//! - [ ] CI/CD
//! - [ ] Improve Transactions - Should return updated object instead of Option<T>
//! - [ ] Implement traits and proc macros to streamline usage - `User.find_many(...)`
//!
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
//! - `all`: [All](database::query::Query::all) - Return all documents.
//! - `associated`: [Associated](database::query::Query::associated) - Find documents based on association.
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

pub use crate::{database::query::Query, deeb::Deeb};
pub use deeb_core::entity::{Entity, EntityName};
pub use deeb_macros::Collection;
