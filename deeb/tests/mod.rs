use anyhow::Error;
use deeb::*;
use deeb_macros::Collection;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Collection, Serialize, Deserialize, PartialEq, Debug)]
#[deeb(
    name = "comment", 
    primary_key = "id",
    associate = ("user", "user_id", "id"),
)]
struct Comment {
    user_id: i32,
    comment: String,
}

#[derive(Collection, Serialize, Deserialize, PartialEq, Debug)]
#[deeb(
    name = "user",
    primary_key = "id",
    associate = ("comment", "id", "user_id", "user_comment"),
)]
struct User {
    id: i32,
    name: String,
    age: f32,
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

#[derive(Collection, Deserialize, Serialize, Clone, Debug)]
#[allow(dead_code)]
#[deeb(name = "user_address")]
struct UserAddress {
    name: String,
    address: Address,
}

async fn spawn_deeb(instance_name: &str) -> Result<(Deeb, Entity, Entity, Entity), Error> {
    let db = Deeb::new();

    let user = User::entity();
    let comment = Comment::entity();
    let user_address = UserAddress::entity();

    // Add instances
    db.add_instance(
        instance_name,
        &format!("./db/test_{}.json", instance_name),
        vec![user.clone(), comment.clone(), user_address.clone()],
    )
    .await?;

    db.delete_many(&user, Query::All, None).await?;
    db.delete_many(&comment, Query::All, None).await?;

    // Populate initial data
    db.insert_one::<User>(
        &user,
        User {
            id: 1,
            name: "oliver".to_string(),
            age: 0.5,
        },
        None,
    )
    .await?;
    db.insert_one::<User>(
        &user,
        User {
            id: 2,
            name: "magnolia".to_string(),
            age: 0.5,
        },
        None,
    )
    .await?;
    db.insert_one::<User>(
        &user,
        User {
            id: 3,
            name: "olliard".to_string(),
            age: 0.5,
        },
        None,
    )
    .await?;

    db.insert_one::<Comment>(
        &comment,
        Comment {
            user_id: 1,
            comment: "Hello".to_string(),
        },
        None,
    )
    .await?;
    db.insert_one::<Comment>(
        &comment,
        Comment {
            user_id: 1,
            comment: "Hi".to_string(),
        },
        None,
    )
    .await?;
    db.insert_one::<Comment>(
        &comment,
        Comment {
            user_id: 2,
            comment: "Hey".to_string(),
        },
        None,
    )
    .await?;
    db.insert_one::<Comment>(
        &comment,
        Comment {
            user_id: 3,
            comment: "Hola".to_string(),
        },
        None,
    )
    .await?;

    Ok((db, user, comment, user_address))
}

#[tokio::test]
async fn insert_one() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("insert_one").await?;
    let value = User {
        id: 12345,
        name: "nick".to_string(),
        age: 35.0,
    };
    let result = db.insert_one::<User>(&user, value, None).await?;
    assert_eq!(
        result,
        serde_json::from_value::<User>(json!({"name": "nick", "age": 35, "id": 12345}))?
    );
    Ok(())
}

#[tokio::test]
async fn insert_one_macro() -> Result<(), Error> {
    let (db, ..) = spawn_deeb("insert_one_macro").await?;
    let value = User {
        id: 12345,
        name: "nick".to_string(),
        age: 35.0,
    };
    let result = User::insert_one(&db, value, None).await?;
    assert_eq!(
        result,
        serde_json::from_value::<User>(json!({"name": "nick", "age": 35, "id": 12345}))?
    );
    Ok(())
}

#[tokio::test]
async fn insert_many() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("insert_many").await?;
    let values = vec![
        User {
            name: "jack".to_string(),
            age: 21.0,
            id: 92884,
        },
        User {
            name: "jull".to_string(),
            age: 20.0,
            id: 923489,
        },
    ];
    let result = db.insert_many::<User>(&user, values, None).await?;
    let expected = vec![
        User {
            name: "jack".to_string(),
            age: 21.0,
            id: 92884,
        },
        User {
            name: "jull".to_string(),
            age: 20.0,
            id: 923489,
        },
    ];
    assert_eq!(result, expected);
    Ok(())
}

#[tokio::test]
async fn insert_many_macro() -> Result<(), Error> {
    let (db, ..) = spawn_deeb("insert_many_macro").await?;
    let values = vec![
        User {
            name: "jack".to_string(),
            age: 21.0,
            id: 92884,
        },
        User {
            name: "jull".to_string(),
            age: 20.0,
            id: 923489,
        },
    ];
    let result = User::insert_many(&db, values, None).await?;
    let expected = vec![
        User {
            name: "jack".to_string(),
            age: 21.0,
            id: 92884,
        },
        User {
            name: "jull".to_string(),
            age: 20.0,
            id: 923489,
        },
    ];
    assert_eq!(result, expected);
    Ok(())
}

#[tokio::test]
async fn find_one() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("find_one").await?;
    let query = Query::eq("name", "oliver");
    let result = db.find_one::<User>(&user, query, None).await?;
    assert_eq!(
        Some(User {
            id: 1,
            name: "oliver".to_string(),
            age: 0.5
        }),
        result
    );
    Ok(())
}

#[tokio::test]
async fn find_one_macro() -> Result<(), Error> {
    let (db, ..) = spawn_deeb("find_one_macro").await?;
    let query = Query::eq("name", "oliver");
    let result = User::find_one(&db, query, None).await?;
    assert_eq!(
        Some(User {
            id: 1,
            name: "oliver".to_string(),
            age: 0.5
        }),
        result
    );
    Ok(())
}

#[tokio::test]
async fn find_many() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("find_many").await?;
    let query = Query::eq("age", 0.5);
    let result = db
        .find_many::<User>(&user, query, None, None)
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
async fn find_many_macro() -> Result<(), Error> {
    let (db, ..) = spawn_deeb("find_many_macro").await?;
    let query = Query::eq("age", 0.5);
    let result = User::find_many(&db, query, None, None).await?.unwrap();
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
async fn find_many_with_limit() -> Result<(), Error> {
    let (db, user, ..) = spawn_deeb("find_many_with_limit").await?;
    let query = Query::eq("age", 0.5);
    let options = Some(FindManyOptions {
        limit: Some(2),
        skip: None,
        order: None,
    });

    let result = db
        .find_many::<User>(&user, query, options, None)
        .await?
        .ok_or_else(|| Error::msg("Expected Users but found none"))?;

    assert_eq!(result.len(), 2);
    Ok(())
}

#[tokio::test]
async fn find_many_with_skip() -> Result<(), Error> {
    let (db, user, ..) = spawn_deeb("find_many_with_skip").await?;
    let query = Query::eq("age", 0.5);
    let options = Some(FindManyOptions {
        limit: None,
        skip: Some(1),
        order: None,
    });

    let result = db
        .find_many::<User>(&user, query, options, None)
        .await?
        .ok_or_else(|| Error::msg("Expected Users but found none"))?;

    assert_eq!(result.len(), 2);
    Ok(())
}

#[tokio::test]
async fn find_many_with_limit_and_skip() -> Result<(), Error> {
    let (db, user, ..) = spawn_deeb("find_many_with_limit_and_skip").await?;
    let query = Query::eq("age", 0.5);
    let options = Some(FindManyOptions {
        limit: Some(1),
        skip: Some(1),
        order: None,
    });

    let result = db
        .find_many::<User>(&user, query, options, None)
        .await?
        .ok_or_else(|| Error::msg("Expected Users but found none"))?;

    assert_eq!(result.len(), 1);
    Ok(())
}

#[tokio::test]
async fn find_many_with_ordering() -> Result<(), Error> {
    let (db, user, ..) = spawn_deeb("find_many_with_ordering").await?;
    let query = Query::eq("age", 0.5);
    let options = Some(FindManyOptions {
        limit: None,
        skip: None,
        order: Some(vec![FindManyOrder {
            property: "name".to_string(),
            direction: OrderDirection::Ascending,
        }]),
    });

    let result = db
        .find_many::<User>(&user, query, options, None)
        .await?
        .ok_or_else(|| Error::msg("Expected Users but found none"))?;

    let names: Vec<String> = result.iter().map(|u| u.name.clone()).collect();

    assert_eq!(names, vec!["magnolia", "oliver", "olliard"]);
    Ok(())
}

#[tokio::test]
async fn delete_one() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("delete_one").await?;
    let query = Query::eq("name", "oliver");
    let result = db
        .delete_one(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected delete result but found none."))?;

    assert_eq!(result, true);
    Ok(())
}

#[tokio::test]
async fn delete_one_macro() -> Result<(), Error> {
    let (db, ..) = spawn_deeb("delete_one_macro").await?;
    let query = Query::eq("name", "oliver");
    let result = User::delete_one(&db, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected delete result but found none."))?;
    assert_eq!(result, true);
    Ok(())
}

#[tokio::test]
async fn delete_many() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("delete_many").await?;
    let query = Query::eq("age", 0.5);
    let result = db
        .delete_many(&user, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected delete result but found none."))?;
    assert!(result);
    Ok(())
}

#[tokio::test]
async fn delete_many_macro() -> Result<(), Error> {
    let (db, ..) = spawn_deeb("delete_many_macro").await?;
    let query = Query::eq("age", 0.5);
    let result = User::delete_many(&db, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected delete result but found none."))?;
    assert!(result);
    Ok(())
}

#[tokio::test]
async fn transaction() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("transcation").await?;
    let mut transaction = db.begin_transaction().await;
    db.insert_one::<User>(
        &user,
        User {
            name: "Al".to_string(),
            age: 45.0,
            id: 255,
        },
        Some(&mut transaction),
    )
    .await?;
    db.insert_one::<User>(
        &user,
        User {
            name: "Peg".to_string(),
            age: 40.0,
            id: 256,
        },
        Some(&mut transaction),
    )
    .await?;
    db.insert_one::<User>(
        &user,
        User {
            name: "Bud".to_string(),
            age: 18.0,
            id: 257,
        },
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
        .find_many::<User>(&user, query, None, None)
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
async fn transaction_macro() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("transaction_macro").await?;
    let mut transaction = db.begin_transaction().await;
    User::insert_one(
        &db,
        User {
            name: "Al".to_string(),
            age: 45.0,
            id: 255,
        },
        Some(&mut transaction),
    )
    .await?;
    User::insert_one(
        &db,
        User {
            name: "Peg".to_string(),
            age: 40.0,
            id: 256,
        },
        Some(&mut transaction),
    )
    .await?;
    User::insert_one(
        &db,
        User {
            name: "Bud".to_string(),
            age: 18.0,
            id: 257,
        },
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
        .find_many::<User>(&user, query, None, None)
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

#[derive(Serialize)]
struct UpdateUser {
    name: Option<String>,
    age: Option<f32>,
}

#[tokio::test]
async fn update_one() -> Result<(), Error> {
    let (db, user, ..) = spawn_deeb("update_one").await?;
    let query = Query::eq("name", "oliver");
    let update = UpdateUser {
        name: Some("olivia".to_string()),
        age: None,
    };
    let result = db
        .update_one::<User, UpdateUser>(&user, query, update, None)
        .await?;
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
async fn update_one_macro() -> Result<(), Error> {
    let (db, ..) = spawn_deeb("update_one_macro").await?;
    let query = Query::eq("name", "oliver");
    let update = UpdateUser {
        name: Some("olivia".to_string()),
        age: None,
    };
    let result = User::update_one(&db, query, update, None).await?;
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
    let (db, user, _comment, ..) = spawn_deeb("update_many").await?;
    let query = Query::eq("age", 0.5);
    let update = UpdateUser {
        age: Some(1.0),
        name: None,
    };
    let result = db
        .update_many::<User, UpdateUser>(&user, query, update, None)
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

#[tokio::test]
async fn update_many_macro() -> Result<(), Error> {
    let (db, ..) = spawn_deeb("update_many_macro").await?;
    let query = Query::eq("age", 0.5);
    let update = UpdateUser {
        age: Some(1.0),
        name: None,
    };
    let result = User::update_many(&db, query, update, None)
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(dead_code)]
struct UserWithoutAge {
    id: i32,
    name: String,
}

#[tokio::test]
async fn drop_key() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("drop_key").await?;
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

#[tokio::test]
async fn drop_key_nested() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("drop_key_nested").await?;
    db.delete_many(&user, Query::All, None).await?;
    db.insert_one::<UserAddress>(
        &user,
        UserAddress {
            name: "oliver".to_string(),
            address: Address {
                city: "lagos".to_string(),
                country: "nigeria".to_string(),
                meta: Some(AddressMeta {
                    zip: 10001,
                    additional: Some("info".to_string()),
                }),
            },
        },
        None,
    )
    .await?;
    db.insert_one::<UserAddress>(
        &user,
        UserAddress {
            name: "olivia".to_string(),
            address: Address {
                city: "lagos".to_string(),
                country: "nigeria".to_string(),
                meta: Some(AddressMeta {
                    zip: 10001,
                    additional: Some("info".to_string()),
                }),
            },
        },
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
    let (db, user, _comment, ..) = spawn_deeb("add_key").await?;
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

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
struct UserAddressBefore {
    name: String,
    address: Option<Address>,
}

#[tokio::test]
async fn add_key_nested() -> Result<(), Error> {
    let (db, _user, _comment, user_address) = spawn_deeb("add_key_nested").await?;
    db.delete_many(&user_address, Query::All, None).await?;
    db.insert_one::<UserAddress>(
        &user_address,
        UserAddress {
            name: "oliver".to_string(),
            address: Address {
                city: "lagos".to_string(),
                country: "nigeria".to_string(),
                meta: None,
            },
        },
        None,
    )
    .await?;
    db.insert_one::<UserAddress>(
        &user_address,
        UserAddress {
            name: "oliver".to_string(),
            address: Address {
                city: "lagos".to_string(),
                country: "nigeria".to_string(),
                meta: None,
            },
        },
        None,
    )
    .await?;

    db.insert_one::<UserAddressBefore>(
        &user_address,
        UserAddressBefore {
            name: "olivia".to_string(),
            address: None,
        },
        None,
    )
    .await?;

    db.add_key(&user_address, "address.meta.zip", 12222).await?;
    db.add_key(&user_address, "address.meta.additional", "Yo")
        .await?;

    let query = Query::eq("address.meta.zip", 12222);
    let result = db
        .find_one::<UserAddress>(&user_address, query, None)
        .await?
        .ok_or_else(|| Error::msg("Expected type but found none."))?;
    assert_eq!(result.address.meta.unwrap().zip, 12222);
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
struct UserWithComments {
    id: i32,
    name: String,
    age: f32,
    user_comment: Vec<Comment>,
}

#[tokio::test]
async fn find_by_association() -> Result<(), Error> {
    let (db, user, comment, ..) = spawn_deeb("find_by_association").await?;
    let query = Query::associated(comment.clone(), Query::eq("user_comment.comment", "Hello"));
    let result = db
        .find_many::<UserWithComments>(&user, query, None, None)
        .await?
        .ok_or_else(|| Error::msg("Expected type but found none."))?;
    let first_comment = result[0].user_comment[0].comment.clone();
    assert_eq!(first_comment, "Hello");
    Ok(())
}
