# Deeb - JSON Database

Prounced how you like, Deeb is an Acid(ish) Compliant JSON based database for small 
websites and fast prototyping. 

Inspired by flexibility of Mongo and light weight of SqLite, Deeb is a tool 
that turns a set of JSON files into a light weight database. 

Deeb's ability to turn groups JSON files into a database allows you to simply 
open a json file and edit as needed.

Check out the quick start below, or the [docs](https://docs.rs/deeb/latest/deeb/) 
to learn more.

## Quick Start

1. Add Deeb to your `Cargo.toml` file

```bash
cargo add deeb
```

2. Optionally, Create a JSON file for your database. Deeb will also create one for you if you like. 

```bash
echo '{"user": []}' > user.json
echo '{"comment": []}' > comment.json
```

**Terminology**
- Instance: A single .json file managed by Deeb. Each instance can store multiple entities and serves as a lightweight, self-contained database.
- Entity: Similar to a table (SQL) or collection (MongoDB), an entity groups documents of a consistent type within an instance.
- Document: An individual record within an entity. Documents are stored as JSON objects and represent a single unit of data (e.g., a user, message, or task).

3. Create a deeb instance and perform operations.

```rust
use deeb::*;
use serde_json::json;
use anyhow::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
     // Set up a new Deeb instance
    let db = Deeb::new();

    // Create a new entity
    let user = Entity::new("user");
    let comment = Entity::new("comment");

    db.add_instance("test", "./user.json", vec![user.clone()])
        .await?;
    db.add_instance("test2", "./comment.json", vec![comment.clone()])
        .await?;

    // Single Operations
    db.insert(&user, json!({"id": 1, "name": "Joey", "age": 10}), None).await?;
    db.find_one(&user, Query::eq("name", "Joey"), None).await?;

    // Perform a transaction
    let mut transaction = db.begin_transaction().await;

    // Insert data into the database
    db.insert(&user, json!({"id": 1, "name": "Steve", "age": 3}), Some(&mut transaction)).await?;
    db.insert(&user, json!({"id": 2, "name": "Johnny", "age": 3}), Some(&mut transaction)).await?;

    db.insert(&comment, json!({"user_id": 1, "comment": "Hello"}), Some(&mut transaction)).await?;
    db.insert(&comment, json!({"user_id": 1, "comment": "Hi"}), Some(&mut transaction)).await?;

    // Query the database
    let query = Query::like("name", "Steve");
    let result = db.find_one(&user, query, Some(&mut transaction)).await?;

    // Update the database
    let query = Query::ne("name", "Steve");
    let update = json!({"name": "Steve", "age": 3});
    db.update_one(&user, query, update, Some(&mut transaction)).await?;

    // Delete from the database
    let query = Query::eq("name", "Johnny");
    db.delete_one(&user, query, Some(&mut transaction)).await?;

    db.commit(&mut transaction).await?;

    Ok(())
}
```

## Features

- **ACIDish Compliant**: Deeb is an ACIDish compliant database. We get close as we can for a light weight JSON based DB.
- **JSON-Based Storage**: Deeb uses lightweight JSON files as the underlying data store, providing human-readable structure and seamless integration with any system that speaks JSON.
- **Schemaless**: Deeb doesn't require a predefined schema like traditional SQL or strongly-typed NoSQL databases. However, by using Rust generics, you can enforce type safety at compile time. This means Deeb stays flexible at runtime, while giving you confidence at build time.
- **Transactions**: Perform multiple operations as a single unit — commit them all at once or roll them back if something fails.
- **Querying**: Deeb supports querying, nested queries, and combination queries.

## Roadmap

- [x] Basic CRUD Operations
- [x] Transactions
- [ ] Indexing
- [x] Querying
- [ ] Migrations
- [x] Benchmarks
- [x] Associations
- [x] Documentation
- [x] Tests
- [x] Examples
- [ ] Logging
- [ ] Error Handling
- [ ] CI/CD
- [ ] Improve Transactions - Should return updated object instead of Option<T>
- [ ] Implement traits and proc macros to streamline usage - `User.find_many(...)`

