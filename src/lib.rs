// use crate::database::entity::Entity;
// use database::query::Query;
// use deeb::Deeb;
// use serde_json::json;

mod database;
mod deeb;

pub use crate::{
    database::{entity::Entity, query::Query},
    deeb::Deeb,
};

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     env_logger::init();

//     let test = Entity::from("test");

//     // Set up the database
//     let db = Deeb::new();

//     db.add_instance("test", "./test.json", vec!["test".into()])
//         .await?;

//     // let mut transaction = db.begin_transaction().await;
//     // db.insert(&test, json!({"test": "test"}), Some(&mut transaction))
//     //     .await?;
//     // db.insert(&test, json!({"test": "test2"}), Some(&mut transaction))
//     //     .await?;
//     // db.find_one(&test, json!({}), Some(&mut transaction))
//     //     .await?;
//     // db.commit(&mut transaction).await?;

//     // db.insert(
//     //     &test,
//     //     json!({"test": "test2", "name": "laura", "age": 35}),
//     //     None,
//     // )
//     // .await?;

//     // db.insert(
//     //     &test,
//     //     json!({"test": "test2", "name": "bongo", "age": 75}),
//     //     None,
//     // )
//     // .await?;

//     let query = Query::or(vec![
//         Query::Eq("name".into(), "nick".into()),
//         Query::Lt("age".into(), 35.into()),
//     ]);
//     let result = db.find_many(&test, query.clone(), None).await?;
//     println!("{:?}", result);

//     let mut transaction = db.begin_transaction().await;

//     db.insert(
//         &test,
//         json!({"test": "test2", "name": "Oaks", "age": 300}),
//         Some(&mut transaction),
//     )
//     .await?;

//     db.insert(
//         &test,
//         json!({"test": "test2", "name": "Ollie", "age": 3}),
//         Some(&mut transaction),
//     )
//     .await?;

//     db.insert(
//         &test,
//         json!({"test": "test", "name": "nick", "age": 35}),
//         Some(&mut transaction),
//     )
//     .await?;

//     db.delete_many(&test, query.clone(), Some(&mut transaction))
//         .await?;

//     let query = Query::Gt("age".into(), 20.into());
//     let res = db.find_many(&test, query, Some(&mut transaction)).await?;
//     println!("{:?}", res);

//     db.commit(&mut transaction).await?;

//     // println!("{:?}", res);

//     Ok(())
// }
