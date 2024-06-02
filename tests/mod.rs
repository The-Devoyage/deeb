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

#[tokio::test]
async fn transaction() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::from("user");
    let mut transaction = db.begin_transaction().await;
    db.insert(
        &entity,
        json!({"name": "Al", "age": 45}),
        Some(&mut transaction),
    )
    .await?;
    db.insert(
        &entity,
        json!({"name": "Peg", "age": 40}),
        Some(&mut transaction),
    )
    .await?;
    db.insert(
        &entity,
        json!({"name": "Bud", "age": 18}),
        Some(&mut transaction),
    )
    .await?;
    db.commit(&mut transaction).await?;
    let query = Query::Or(vec![
        Query::Eq("name".into(), "Al".into()),
        Query::Eq("name".into(), "Peg".into()),
        Query::Eq("name".into(), "Bud".into()),
    ]);
    let result = db.find_many(&entity, query, None).await?;
    assert!(
        result.contains(&json!({"name": "Al", "age": 45}))
            && result.contains(&json!({"name": "Peg", "age": 40}))
            && result.contains(&json!({"name": "Bud", "age": 18}))
    );
    Ok(())
}

#[tokio::test]
async fn test_eq() {
    let query = Query::Eq("name".into(), "nick".into());
    let value = json!({"name": "nick", "age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_eq() {
    let query = Query::Eq("user.name".into(), "nick".into());
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_ne() {
    let query = Query::Ne("name".into(), "nick".into());
    let value = json!({"name": "nick", "age": 35});
    assert!(!query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_ne() {
    let query = Query::Ne("user.name".into(), "nick".into());
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(!query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_like() {
    let query = Query::Like("name".into(), "ni".into());
    let value = json!({"name": "nick", "age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_like() {
    let query = Query::Like("user.name".into(), "ni".into());
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_lt() {
    let query = Query::Lt("age".into(), 35.into());
    let value = json!({"age": 34});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_lt() {
    let query = Query::Lt("user.age".into(), 35.into());
    let value = json!({"user": {"name": "nick", "age": 34}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_lte() {
    let query = Query::Lte("age".into(), 35.into());
    let value = json!({"age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_lte() {
    let query = Query::Lte("user.age".into(), 35.into());
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_gt() {
    let query = Query::Gt("age".into(), 35.into());
    let value = json!({"age": 36});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_gt() {
    let query = Query::Gt("user.age".into(), 35.into());
    let value = json!({"user": {"name": "nick", "age": 36}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_gte() {
    let query = Query::Gte("age".into(), 35.into());
    let value = json!({"age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_gte() {
    let query = Query::Gte("user.age".into(), 35.into());
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_and() {
    let query = Query::And(vec![
        Query::Eq("name".into(), "nick".into()),
        Query::Lt("age".into(), 35.into()),
    ]);
    let value = json!({"name": "nick", "age": 34});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_or() {
    let query = Query::Or(vec![
        Query::Eq("name".into(), "nick".into()),
        Query::Lt("age".into(), 35.into()),
    ]);
    let value = json!({"name": "nick", "age": 36});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_all() {
    let query = Query::All;
    let value = json!({"name": "nick", "age": 35});
    assert!(query.matches(&value).unwrap());
}
