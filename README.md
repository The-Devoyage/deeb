# Deeb - JSON Database

Prounced `D-B`, Deeb is an Acid Compliant JSON based database for small 
websites and fast prototyping. 

Inspired by flexibility of Mongo and light weight of SqLite, Deeb is a tool 
that turns a set of JSON files into a database. 

Deeb's ability to turn groups JSON files into a database allows you to simply 
open a json file and edit as needed.

Check out the quick start below, or the [docs](https://docs.rs/deeb/latest/deeb/) 
to learn more.

 ## Quick Start

```rust
use deeb::*;
use serde_json::json;
use anyhow::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
     // Set up a new Deeb instance
    let db = Deeb::new();

    // Create a new entity
    let user = Entity::from("user");
    let comment = Entity::from("comment");

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

- **ACID Compliant**: Deeb is an ACID compliant database
- **JSON Based**: Deeb uses JSON files as the database
- **Schemaless**: Deeb is schemaless
- **Transactions**: Deeb supports transactions
- **Querying**: Deeb supports querying, nested queries, and combination queries.

## Roadmap

- [x] Basic CRUD Operations
- [x] Transactions
- [ ] Indexing
- [x] Querying
- [ ] Migrations
- [x] Benchmarks
- [x] Documentation
- [x] Tests
- [ ] Examples
- [ ] Logging
- [ ] Error Handling
- [ ] CI/CD

