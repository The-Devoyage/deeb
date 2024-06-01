// use clap::Parser;
// use cli::Cli;
use crate::database::entity::Entity;
use deeb::Deeb;
use serde_json::json;

mod cli;
pub mod database;
mod deeb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let cli = Cli::parse();

    let test = Entity::from("test");

    // Set up the database
    let db = Deeb::new();

    db.add_instance("test", "./test.json", vec!["test".into()])
        .await?;

    // let mut transaction = db.begin_transaction().await;
    // db.insert(&test, json!({"test": "test"}), Some(&mut transaction))
    //     .await?;
    // db.insert(&test, json!({"test": "test2"}), Some(&mut transaction))
    //     .await?;
    // db.find_one(&test, json!({}), Some(&mut transaction))
    //     .await?;
    // db.commit(&mut transaction).await?;

    db.insert(&test, json!({"test": "test"}), None).await?;
    db.insert(&test, json!({"test": "test2"}), None).await?;
    db.find_one(&test, json!({}), None).await?;

    Ok(())
}
