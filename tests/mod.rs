use anyhow::Error;
use deeb::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, PartialEq, Debug)]
struct User {
    id: i32,
    name: String,
    age: f32,
}

#[derive(Deserialize, PartialEq, Debug)]
struct Comment {
    user_id: i32,
    comment: String,
}

async fn spawn_deeb() -> Result<(Deeb, Entity, Entity), Error> {
    let db = Deeb::new();

    // Define entities
    let mut comment = Entity::new("comment").primary_key("id");
    let user = Entity::new("user")
        .primary_key("id")
        .associate(&mut comment, "user_id", Some("user_comment"))
        .map_err(|e| anyhow::anyhow!(e))?;

    // Add instances
    db.add_instance(
        "user",
        "./tests/test.json",
        vec![user.clone(), comment.clone()],
    )
    .await?;

    db.delete_many(&user, Query::All, None).await?;
    db.delete_many(&comment, Query::All, None).await?;

    // Populate initial data
    db.insert::<User>(&user, json!({"id": 1, "name": "oliver", "age": 0.5}), None)
        .await?;
    db.insert::<User>(
        &user,
        json!({"id": 2, "name": "magnolia", "age": 0.5}),
        None,
    )
    .await?;
    db.insert::<User>(&user, json!({"id": 3, "name": "olliard", "age": 0.5}), None)
        .await?;

    db.insert::<Comment>(&comment, json!({"user_id": 1, "comment": "Hello"}), None)
        .await?;
    db.insert::<Comment>(&comment, json!({"user_id": 1, "comment": "Hi"}), None)
        .await?;
    db.insert::<Comment>(&comment, json!({"user_id": 2, "comment": "Hey"}), None)
        .await?;
    db.insert::<Comment>(&comment, json!({"user_id": 3, "comment": "Hola"}), None)
        .await?;

    Ok((db, user, comment))
}

#[tokio::test]
async fn insert_one() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let value = json!({"id": 12345, "name": "nick", "age": 35});
    let result = db.insert::<User>(&user, value, None).await?;
    assert_eq!(
        result,
        serde_json::from_value::<User>(json!({"name": "nick", "age": 35, "id": 12345}))?
    );
    Ok(())
}

#[tokio::test]
async fn insert_many() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let values = vec![
        json!({"name": "jack", "age": 21, "id": 92884}),
        json!({"name": "jull", "age": 20, "id": 923489}),
    ];
    let result = db.insert_many::<User>(&user, values, None).await?;
    let expected: Result<Vec<User>, _> = vec![
        json!({"name": "jack", "age": 21, "id": 92884}),
        json!({"name": "jull", "age": 20, "id": 923489}),
    ]
    .into_iter()
    .map(serde_json::from_value)
    .collect();
    assert_eq!(result, expected?);
    Ok(())
}

#[tokio::test]
async fn find_one() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let query = Query::eq("name", "oliver");
    let result = db.find_one::<User>(&user, query, None).await?;
    assert_eq!(
        result,
        Some(serde_json::from_value::<User>(
            json!({"id": 1,"name": "oliver", "age": 0.5})
        )?)
    );
    Ok(())
}

#[tokio::test]
async fn find_many() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let query = Query::eq("age", 0.5);
    let result = db
        .find_many::<User>(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected Users but found none"))?;

    assert!(
        result.contains(&User {
            id: 1,
            name: "oliver".into(),
            age: 0.5
        }) && result.contains(&User {
            id: 2,
            name: "magnolia".into(),
            age: 0.5
        }) && result.contains(&User {
            id: 3,
            name: "olliard".into(),
            age: 0.5
        })
    );

    Ok(())
}

#[tokio::test]
async fn delete_one() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let query = Query::eq("name", "oliver");
    let result = db
        .delete_one(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected delete result but found none."))?;

    assert_eq!(result, true);
    Ok(())
}

#[tokio::test]
async fn delete_many() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let query = Query::eq("age", 0.5);
    let result = db
        .delete_many(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected delete result but found none."))?;
    assert!(result);
    Ok(())
}

#[tokio::test]
async fn transaction() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let mut transaction = db.begin_transaction().await;
    db.insert::<User>(
        &user,
        json!({"name": "Al", "age": 45.0, "id": 255}),
        Some(&mut transaction),
    )
    .await?;
    db.insert::<User>(
        &user,
        json!({"name": "Peg", "age": 40.0, "id": 256}),
        Some(&mut transaction),
    )
    .await?;
    db.insert::<User>(
        &user,
        json!({"name": "Bud", "age": 18.0, "id": 257}),
        Some(&mut transaction),
    )
    .await?;
    db.commit(&mut transaction).await?;
    let query = Query::Or(vec![
        Query::eq("name", "Al"),
        Query::eq("name", "Peg"),
        Query::eq("name", "Bud"),
    ]);
    let result = db
        .find_many::<User>(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected vec of type but found none."))?;
    assert!(
        result.contains(&User {
            name: "Al".to_string(),
            age: 45.0,
            id: 255
        }) && result.contains(&User {
            name: "Peg".to_string(),
            age: 40.0,
            id: 256
        }) && result.contains(&User {
            name: "Bud".to_string(),
            age: 18.0,
            id: 257
        })
    );
    Ok(())
}

#[tokio::test]
async fn update_one() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let query = Query::eq("name", "oliver");
    let update = json!({"name": "olivia"});
    let result = db.update_one::<User>(&user, query, update, None).await?;
    assert_eq!(
        result,
        Some(User {
            id: 1,
            name: "olivia".to_string(),
            age: 0.5
        })
    );
    Ok(())
}

#[tokio::test]
async fn update_many() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    let query = Query::eq("age", 0.5);
    let update = json!({"age": 1.0});
    let result = db
        .update_many(&user, query, update, None)
        .await?
        .ok_or_else(|| Error::msg("Expected vector but received none."))?;
    assert!(
        result.contains(&User {
            id: 1,
            name: "oliver".into(),
            age: 1.0
        }) && result.contains(&User {
            id: 2,
            name: "magnolia".into(),
            age: 1.0
        }) && result.contains(&User {
            id: 3,
            name: "olliard".into(),
            age: 1.0
        })
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
async fn test_array_eq() {
    let query = Query::eq("names", "nick");
    let value = json!({ "names": ["jones", "nick", "olliard", "magnolia"] });
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_array_object_eq() {
    let query = Query::eq("user.name", "nick");
    let value = json!({"user": [{"name": "jones", "age": 25}, {"name": "nick", "age": 35}]});
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
    let is_match = query.matches(&value).unwrap();
    println!("{:?}", is_match);
    assert!(!query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_array_ne() {
    let query = Query::ne("names", "nick");
    let value = json!({ "names": ["jones", "olliard", "magnolia"] });
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_array_object_ne() {
    let query = Query::ne("user.name", "nick");
    let value = json!({"user": [{"name": "jimmy", "age": 35}, {"name": "nick", "age": 35}]});
    assert!(query.matches(&value).unwrap());
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
async fn test_array_like() {
    let query = Query::like("names", "ni");
    let value = json!({ "names": ["jack", "nick", "olliard", "magnolia"] });
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_array_object_like() {
    let query = Query::like("user.name", "ni");
    let value = json!({"user": [{"name": "noodle", "age": 35}, {"name": "nick", "age": 35}]});
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
async fn test_array_lt() {
    let query = Query::lt("ages", 35);
    let value = json!({ "ages": [39, 34, 36, 37] });
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_array_object_lt() {
    let query = Query::lt("user.age", 35);
    let value = json!({"user": [{"name": "nick", "age": 39}, {"name": "nick", "age": 34}]});
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
async fn test_array_lte() {
    let query = Query::lte("ages", 35);
    let value = json!({ "ages": [44, 34, 35, 37] });
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_array_object_lte() {
    let query = Query::lte("user.age", 35);
    let value = json!({"user": [{"name": "nick", "age": 39}, {"name": "nick", "age": 35}]});
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
async fn test_array_gt() {
    let query = Query::gt("ages", 35);
    let value = json!({ "ages": [34, 36, 37] });
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_array_object_gt() {
    let query = Query::gt("user.age", 35);
    let value = json!({"user": [{"name": "nick", "age": 36}]});
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
async fn test_array_gte() {
    let query = Query::gte("ages", 35);
    let value = json!({ "ages": [34, 35, 37] });
    assert!(query.matches(&value).unwrap());
}

#[tokio::test]
async fn test_array_object_gte() {
    let query = Query::gte("user.age", 35);
    let value = json!({"user": [{"name": "nick", "age": 35}]});
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

#[derive(Deserialize, Debug, PartialEq)]
#[allow(dead_code)]
struct UserWithoutAge {
    id: i32,
    name: String,
}

#[tokio::test]
async fn drop_key() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    db.drop_key(&user, "age").await?;
    let query = Query::eq("name", "oliver");
    let result = db
        .find_one::<UserWithoutAge>(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected type but found none."))?;
    assert_eq!(
        result,
        UserWithoutAge {
            id: 1,
            name: "oliver".to_string(),
        }
    );
    Ok(())
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[allow(dead_code)]
struct AddressMeta {
    zip: i32,
    additional: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[allow(dead_code)]
struct Address {
    city: String,
    country: String,
    meta: Option<AddressMeta>,
}

#[derive(Deserialize, Serialize, Clone)]
#[allow(dead_code)]
struct UserAddress {
    name: String,
    address: Address,
}

#[tokio::test]
async fn drop_key_nested() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    db.delete_many(&user, Query::All, None).await?;
    db.insert::<UserAddress>(
        &user,
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
    db.insert::<UserAddress>(
        &user,
        json!({
        "name": "olivia",
        "address": {
            "city": "lagos",
            "country": "nigeria",
            "meta": {"zip": 10001, "additional": "info"}
        }}),
        None,
    )
    .await?;
    db.drop_key(&user, "address.meta.additional").await?;
    let query = Query::eq("address.country", "nigeria");
    let result = db
        .find_one::<UserAddress>(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected type but found none"))?;
    assert!(result.address.meta.unwrap().additional.is_none());
    Ok(())
}

#[derive(Deserialize, Debug, PartialEq)]
struct UserStatus {
    id: i32,
    name: String,
    age: f32,
    status: bool,
}

// Test removing key from nested object that does not have nested paths
// TODO: Should skip the operation for that record
#[tokio::test]
async fn add_key() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    db.add_key(&user, "status", true).await?;
    let query = Query::eq("name", "oliver");
    let result = db
        .find_one::<UserStatus>(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected type but found none."))?;
    assert_eq!(
        result,
        UserStatus {
            id: 1,
            name: "oliver".to_string(),
            age: 0.5,
            status: true
        }
    );
    Ok(())
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct UserName {
    name: String,
    address: Option<Address>,
}

#[tokio::test]
async fn add_key_nested() -> Result<(), Error> {
    let (db, user, _comment) = spawn_deeb().await?;
    db.delete_many(&user, Query::All, None).await?;
    db.insert::<UserAddress>(
        &user,
        json!({"name": "oliver", "address": {"city": "lagos", "country": "nigeria"}}),
        None,
    )
    .await?;
    db.insert::<UserAddress>(
        &user,
        json!({"name": "oliver", "address": {"city": "lagos", "country": "nigeria"}}),
        None,
    )
    .await?;
    db.insert::<UserName>(&user, json!({"name": "olivia" }), None)
        .await?;
    db.add_key(&user, "address.meta.zip", 10001).await?;
    db.add_key(&user, "address.meta.additional", "Yo").await?;
    let query = Query::eq("address.meta.zip", 10001);
    let result = db
        .find_one::<UserName>(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected type but found none."))?;
    assert_eq!(result.address.unwrap().meta.unwrap().zip, 10001);
    Ok(())
}

#[tokio::test]
async fn load_meta() -> Result<(), Error> {
    let (db, ..) = spawn_deeb().await?;
    let _meta = db.get_meta()?;
    let meta = db
        .find_many::<Entity>(&_meta, Query::All, None)
        .await?
        .ok_or_else(|| Error::msg("Expected type but found none."))?;

    assert_eq!(meta.len(), 2);
    assert_eq!(meta[0].name, "user".into());
    assert_eq!(meta[1].name, "comment".into());
    // primary key
    assert_eq!(meta[0].primary_key, Some("id".to_string()));
    assert_eq!(meta[1].primary_key, Some("id".to_string()));
    // associations
    assert_eq!(meta[0].associations[0].from, "id");
    assert_eq!(meta[0].associations[0].to, "user_id");
    assert_eq!(meta[1].associations[0].from, "user_id");
    assert_eq!(meta[1].associations[0].to, "id");

    Ok(())
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct UserWithComments {
    id: i32,
    name: String,
    age: f32,
    user_comment: Vec<Comment>,
}

#[tokio::test]
async fn find_by_association() -> Result<(), Error> {
    let (db, user, comment) = spawn_deeb().await?;
    let query = Query::associated(comment.clone(), Query::eq("user_comment.comment", "Hello"));
    let result = db
        .find_many::<UserWithComments>(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected type but found none."))?;
    let first_comment = result[0].user_comment[0].comment.clone();
    assert_eq!(first_comment, "Hello");
    Ok(())
}
