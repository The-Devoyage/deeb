mod cli;
pub mod database;
mod deeb;

// use clap::Parser;
// use cli::Cli;
use deeb::Deeb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    // let cli = Cli::parse();
    let db = Deeb::new();
    println!("Created Deeb");
    db.add_instance("test".into(), "./test.json", vec!["test".into()])
        .await?;
    println!("Added instance");
    let sample = db
        .insert(&"test".into(), serde_json::json!({"test": "test"}))
        .await?;
    println!("Inserted sample");
    println!("{:?}", sample);

    let found = db.find_one(&"test".into(), serde_json::json!({})).await?;
    println!("Found sample");
    println!("{:?}", found);
    Ok(())
}
