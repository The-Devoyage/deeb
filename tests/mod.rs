use anyhow::Error;
use deeb::*;
use serde_json::json;

async fn spawn_deeb() -> Result<Deeb, Error> {
    let db = Deeb::new();
    db.add_instance("test", "./tests/test.json", vec!["user".into()])
        .await?;

    let entity = Entity::from("user");

    db.delete_many(&entity, Query::All, None).await?;

    // Populate initial data
    db.insert(&entity, json!({"name": "oliver", "age": 0.5}), None)
        .await?;
    db.insert(&entity, json!({"name": "magnolia", "age": 0.5}), None)
        .await?;
    db.insert(&entity, json!({"name": "olliard", "age": 0.5}), None)
        .await?;
    Ok(db)
}

#[tokio::test]
async fn test_new_entity() {
    let entity = Entity::from("test");
    assert_eq!(entity, Entity("test".to_string()));
}

#[tokio::test]
async fn insert_one() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::from("user");
    let value = json!({"name": "nick", "age": 35});
    let result = db.insert(&entity, value, None).await?;
    assert_eq!(result, json!({"name": "nick", "age": 35}));
    Ok(())
}

#[tokio::test]
async fn insert_many() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::from("user");
    let values = vec![
        json!({"name": "jack", "age": 21}),
        json!({"name": "jull", "age": 20}),
    ];
    let result = db.insert_many(&entity, values, None).await?;
    assert_eq!(
        result,
        vec![
            json!({"name": "jack", "age": 21}),
            json!({"name": "jull", "age": 20}),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn find_one() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::from("user");
    let query = Query::Eq("name".into(), "oliver".into());
    let result = db.find_one(&entity, query, None).await?;
    assert_eq!(result, json!({"name": "oliver", "age": 0.5}));
    Ok(())
}

#[tokio::test]
async fn find_many() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::from("user");
    let query = Query::Eq("age".into(), 0.5.into());
    let result = db.find_many(&entity, query, None).await?;
    assert!(
        result.contains(&json!({"name": "oliver", "age": 0.5}))
            && result.contains(&json!({"name": "magnolia", "age": 0.5}))
            && result.contains(&json!({"name": "olliard", "age": 0.5}))
    );
    Ok(())
}

#[tokio::test]
async fn delete_one() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::from("user");
    let query = Query::Eq("name".into(), "oliver".into());
    let result = db.delete_one(&entity, query, None).await?;
    assert_eq!(result, json!({"name": "oliver", "age": 0.5}));
    Ok(())
}

#[tokio::test]
async fn delete_many() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::from("user");
    let query = Query::Eq("age".into(), 0.5.into());
    let result = db.delete_many(&entity, query, None).await?;
    assert!(
        result.contains(&json!({"name": "oliver", "age": 0.5}))
            && result.contains(&json!({"name": "magnolia", "age": 0.5}))
            && result.contains(&json!({"name": "olliard", "age": 0.5}))
    );
    Ok(())
}
