use anyhow::Error;
use deeb::*;
use serde_json::json;

async fn spawn_deeb() -> Result<Deeb, Error> {
    let db = Deeb::new();
    let user = Entity::new("user");
    let comment = Entity::new("comment");

    db.add_instance(
        "user",
        "./tests/test.json",
        vec![user.clone(), comment.clone()],
    )
    .await?;

    db.delete_many(&user, Query::All, None).await?;

    // Populate initial data
    db.insert(&user, json!({"id": 1, "name": "oliver", "age": 0.5}), None)
        .await?;
    db.insert(
        &user,
        json!({"id": 2, "name": "magnolia", "age": 0.5}),
        None,
    )
    .await?;
    db.insert(&user, json!({"id": 3, "name": "olliard", "age": 0.5}), None)
        .await?;

    db.insert(&comment, json!({"user_id": 1, "comment": "Hello"}), None)
        .await?;
    db.insert(&comment, json!({"user_id": 1, "comment": "Hi"}), None)
        .await?;
    db.insert(&comment, json!({"user_id": 2, "comment": "Hey"}), None)
        .await?;
    db.insert(&comment, json!({"user_id": 3, "comment": "Hola"}), None)
        .await?;

    Ok(db)
}

#[tokio::test]
async fn insert_one() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    let value = json!({"name": "nick", "age": 35});
    let result = db.insert(&entity, value, None).await?;
    assert_eq!(result, json!({"name": "nick", "age": 35}));
    Ok(())
}

#[tokio::test]
async fn insert_many() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
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
    let entity = Entity::new("user");
    let query = Query::eq("name", "oliver");
    let result = db.find_one(&entity, query, None).await?;
    assert_eq!(result, json!({"id": 1,"name": "oliver", "age": 0.5}));
    Ok(())
}

#[tokio::test]
async fn find_many() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    let query = Query::eq("age", 0.5);
    let result = db.find_many(&entity, query, None).await?;
    assert!(
        result.contains(&json!({"id": 1, "name": "oliver", "age": 0.5}))
            && result.contains(&json!({"id": 2,"name": "magnolia", "age": 0.5}))
            && result.contains(&json!({"id": 3,"name": "olliard", "age": 0.5}))
    );
    Ok(())
}

#[tokio::test]
async fn delete_one() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    let query = Query::eq("name", "oliver");
    let result = db.delete_one(&entity, query, None).await?;
    assert_eq!(result, json!({"id": 1, "name": "oliver", "age": 0.5}));
    Ok(())
}

#[tokio::test]
async fn delete_many() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    let query = Query::eq("age", 0.5);
    let result = db.delete_many(&entity, query, None).await?;
    assert!(
        result.contains(&json!({"id": 1,"name": "oliver", "age": 0.5}))
            && result.contains(&json!({"id": 2,"name": "magnolia", "age": 0.5}))
            && result.contains(&json!({"id": 3,"name": "olliard", "age": 0.5}))
    );
    Ok(())
}

#[tokio::test]
async fn transaction() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
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
        Query::eq("name", "Al"),
        Query::eq("name", "Peg"),
        Query::eq("name", "Bud"),
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
async fn update_one() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    let query = Query::eq("name", "oliver");
    let update = json!({"name": "olivia"});
    let result = db.update_one(&entity, query, update, None).await?;
    assert_eq!(result, json!({"id": 1,"name": "olivia", "age": 0.5}));
    Ok(())
}

#[tokio::test]
async fn update_many() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    let query = Query::eq("age", 0.5);
    let update = json!({"age": 1.0});
    let result = db.update_many(&entity, query, update, None).await?;
    assert!(
        result.contains(&json!({"id": 1,"name": "oliver", "age": 1.0}))
            && result.contains(&json!({"id": 2,"name": "magnolia", "age": 1.0}))
            && result.contains(&json!({"id": 3,"name": "olliard", "age": 1.0}))
    );
    Ok(())
}

// Test Query
#[tokio::test]
async fn test_eq() {
    let query = Query::eq("name", "nick");
    let value = json!({"name": "nick", "age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_eq() {
    let query = Query::eq("user.name", "nick");
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_ne() {
    let query = Query::ne("name", "nick");
    let value = json!({"name": "nick", "age": 35});
    assert!(!query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_ne() {
    let query = Query::ne("user.name", "nick");
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(!query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_like() {
    let query = Query::like("name", "ni");
    let value = json!({"name": "nick", "age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_like() {
    let query = Query::like("user.name", "ni");
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_lt() {
    let query = Query::lt("age", 35);
    let value = json!({"age": 34});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_lt() {
    let query = Query::lt("user.age", 35);
    let value = json!({"user": {"name": "nick", "age": 34}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_lte() {
    let query = Query::lte("age", 35);
    let value = json!({"age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_lte() {
    let query = Query::lte("user.age", 35);
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_gt() {
    let query = Query::gt("age", 35);
    let value = json!({"age": 36});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_gt() {
    let query = Query::gt("user.age", 35);
    let value = json!({"user": {"name": "nick", "age": 36}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_gte() {
    let query = Query::gte("age", 35);
    let value = json!({"age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_nested_gte() {
    let query = Query::gte("user.age", 35);
    let value = json!({"user": {"name": "nick", "age": 35}});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_and() {
    let query = Query::And(vec![Query::eq("name", "nick"), Query::lt("age", 35)]);
    let value = json!({"name": "nick", "age": 34});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_or() {
    let query = Query::Or(vec![Query::eq("name", "nick"), Query::lt("age", 35)]);
    let value = json!({"name": "nick", "age": 36});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_all() {
    let query = Query::All;
    let value = json!({"name": "nick", "age": 35});
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn drop_key() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    db.drop_key(&entity, "age").await?;
    let query = Query::eq("name", "oliver");
    let result = db.find_one(&entity, query, None).await?;
    assert_eq!(result, json!({"id": 1, "name": "oliver"}));
    Ok(())
}

#[tokio::test]
async fn drop_key_nested() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    db.delete_many(&entity, Query::All, None).await?;
    db.insert(
        &entity,
        json!({
        "name": "oliver",
        "address": {
            "city": "lagos",
            "country": "nigeria",
            "meta": {"zip": 10001, "additional": "info"}
        }}),
        None,
    )
    .await?;
    db.insert(
        &entity,
        json!({
        "name": "olivia",
        "address": {
            "city": "lagos",
            "country": "nigeria",
            "meta": {"zip": 10001, "secondary": "info"}
        }}),
        None,
    )
    .await?;
    db.drop_key(&entity, "address.meta.additional").await?;
    let query = Query::eq("address.country", "nigeria");
    let result = db.find_one(&entity, query, None).await?;
    let result = result.as_object().unwrap();
    let address = result.get("address").unwrap().as_object().unwrap();
    let meta = address.get("meta").unwrap().as_object().unwrap();
    assert_eq!(meta.get("additional"), None);
    Ok(())
}

// Test removing key from nested object that does not have nested paths
// TODO: Should skip the operation for that record

#[tokio::test]
async fn add_key() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    db.add_key(&entity, "status", true).await?;
    let query = Query::eq("name", "oliver");
    let result = db.find_one(&entity, query, None).await?;
    assert_eq!(
        result,
        json!({"id": 1, "name": "oliver", "age": 0.5, "status": true})
    );
    Ok(())
}

#[tokio::test]
async fn add_key_nested() -> Result<(), Error> {
    let db = spawn_deeb().await?;
    let entity = Entity::new("user");
    db.delete_many(&entity, Query::All, None).await?;
    db.insert(
        &entity,
        json!({"name": "oliver", "address": {"city": "lagos", "country": "nigeria"}}),
        None,
    )
    .await?;
    db.insert(
        &entity,
        json!({"name": "oliver", "address": {"city": "lagos", "country": "nigeria"}}),
        None,
    )
    .await?;
    db.insert(&entity, json!({"name": "olivia" }), None).await?;
    db.add_key(&entity, "address.zip", 10001).await?;
    let query = Query::eq("address.zip", 10001);
    let result = db.find_one(&entity, query, None).await?;
    let result = result.as_object().unwrap();
    let address = result.get("address").unwrap().as_object().unwrap();
    assert_eq!(address.get("zip"), Some(&json!(10001)));
    Ok(())
}
