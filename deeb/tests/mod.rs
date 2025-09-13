use anyhow::Error;
use deeb::*;
use deeb_macros::Collection;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Collection, Serialize, Deserialize, PartialEq, Debug)]
#[deeb(name = "product", primary_key = "_id")]
struct Product {
    name: String,
    description: String,
    count: i32,
}

#[derive(Collection, Serialize, Deserialize, PartialEq, Debug)]
#[deeb(
    name = "comment",
    primary_key = "_id",
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

async fn spawn_deeb(instance_name: &str) -> Result<(Deeb, Entity, Entity, Entity, Entity), Error> {
    let db = Deeb::new();

    let user = User::entity();
    let comment = Comment::entity();
    let user_address = UserAddress::entity();
    let mut product = Product::entity();
    product.add_index("product_compound_index", vec!["name", "count"], None)?;
    product.add_index("primary_key_index", vec!["_id"], None)?;

    // Add instances
    db.add_instance(
        instance_name,
        &format!("./db/test_{}.json", instance_name),
        vec![
            user.clone(),
            comment.clone(),
            user_address.clone(),
            product.clone(),
        ],
    )
    .await?;

    db.delete_many(&user, Query::All, None).await?;
    db.delete_many(&comment, Query::All, None).await?;
    db.delete_many(&product, Query::All, None).await?;

    // Populate initial data
    db.insert_one::<User, User>(
        &user,
        User {
            id: 1,
            name: "oliver".to_string(),
            age: 0.5,
        },
        None,
    )
    .await?;
    db.insert_one::<User, User>(
        &user,
        User {
            id: 2,
            name: "magnolia".to_string(),
            age: 0.5,
        },
        None,
    )
    .await?;
    db.insert_one::<User, User>(
        &user,
        User {
            id: 3,
            name: "olliard".to_string(),
            age: 0.5,
        },
        None,
    )
    .await?;

    db.insert_one::<Comment, Comment>(
        &comment,
        Comment {
            user_id: 1,
            comment: "Hello".to_string(),
        },
        None,
    )
    .await?;
    db.insert_one::<Comment, Comment>(
        &comment,
        Comment {
            user_id: 1,
            comment: "Hi".to_string(),
        },
        None,
    )
    .await?;
    db.insert_one::<Comment, Comment>(
        &comment,
        Comment {
            user_id: 2,
            comment: "Hey".to_string(),
        },
        None,
    )
    .await?;
    db.insert_one::<Comment, Comment>(
        &comment,
        Comment {
            user_id: 3,
            comment: "Hola".to_string(),
        },
        None,
    )
    .await?;

    Ok((db, user, comment, user_address, product.clone()))
}

#[tokio::test]
async fn insert_one() -> Result<(), Error> {
    let (db, user, _comment, ..) = spawn_deeb("insert_one").await?;
    let value = User {
        id: 12345,
        name: "nick".to_string(),
        age: 35.0,
    };
    let result = db.insert_one::<User, User>(&user, value, None).await?;
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
    let result = db.insert_many::<User, User>(&user, values, None).await?;
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
    db.insert_one::<User, User>(
        &user,
        User {
            name: "Al".to_string(),
            age: 45.0,
            id: 255,
        },
        Some(&mut transaction),
    )
    .await?;
    db.insert_one::<User, User>(
        &user,
        User {
            name: "Peg".to_string(),
            age: 40.0,
            id: 256,
        },
        Some(&mut transaction),
    )
    .await?;
    db.insert_one::<User, User>(
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
    let (db, _user, _comment, user_address, ..) = spawn_deeb("drop_key_nested").await?;
    db.delete_many(&user_address, Query::All, None).await?;
    db.insert_one::<UserAddress, UserAddress>(
        &user_address,
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
    db.insert_one::<UserAddress, UserAddress>(
        &user_address,
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
    db.drop_key(&user_address, "address.meta.additional")
        .await?;
    let query = Query::eq("address.country", "nigeria");
    let result = db
        .find_one::<UserAddress>(&user_address, query, None)
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
    let (db, _user, _comment, user_address, _product) = spawn_deeb("add_key_nested").await?;
    db.delete_many(&user_address, Query::All, None).await?;
    db.insert_one::<UserAddress, UserAddress>(
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
    db.insert_one::<UserAddress, UserAddress>(
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

    db.insert_one::<UserAddressBefore, UserAddress>(
        &user_address,
        UserAddressBefore {
            name: "olivia".to_string(),
            address: Some(Address {
                city: "new york".to_string(),
                country: "USA".to_string(),
                meta: None,
            }),
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

    // Flatten all comments from all users
    let all_comments: Vec<_> = result
        .iter()
        .flat_map(|user_with_comments| {
            user_with_comments
                .user_comment
                .iter()
                .map(|c| c.comment.clone())
        })
        .collect();

    // Assert that "Hello" is in the comments
    assert!(
        all_comments.contains(&"Hello".to_string()),
        "Expected to find a comment with 'Hello', but got: {:?}",
        all_comments
    );

    Ok(())
}

// Indexes
#[tokio::test]
async fn find_one_with_compound_index() -> Result<(), Error> {
    let (db, _user, _comment, _ua, product) = spawn_deeb("find_one_with_compound_index").await?;
    let values = vec![
        Product {
            name: "keyboard".to_string(),
            description: "Computer keyboard".to_string(),
            count: 2000,
        },
        Product {
            name: "monitor".to_string(),
            description: "Computer monitor".to_string(),
            count: 2000,
        },
        Product {
            name: "mouse".to_string(),
            description: "Computer mouse".to_string(),
            count: 5000,
        },
    ];
    db.insert_many::<Product, Product>(&product, values, None)
        .await?;

    let query = Query::And(vec![Query::eq("name", "mouse"), Query::eq("count", 5000)]);
    let result = db.find_one::<Product>(&product, query, None).await?;

    assert_eq!(
        result,
        Some(Product {
            name: "mouse".to_string(),
            description: "Computer mouse".to_string(),
            count: 5000,
        })
    );

    Ok(())
}

#[tokio::test]
async fn find_one_with_pk_index() -> Result<(), Error> {
    let (db, _user, _comment, _ua, product) = spawn_deeb("find_one_with_pk_index").await?;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ProductWithId {
        _id: String,
        name: String,
        description: String,
        count: i32,
    }

    let p = Product {
        name: "keyboard".to_string(),
        description: "Computer keyboard".to_string(),
        count: 2000,
    };

    let inserted_product = db
        .insert_one::<Product, ProductWithId>(&product, p, None)
        .await?;

    let query = Query::eq("_id", inserted_product._id.clone());
    let found_product = db.find_one::<ProductWithId>(&product, query, None).await?;

    assert_eq!(Some(inserted_product), found_product);

    Ok(())
}

#[tokio::test]
async fn test_save_instance_config_default_path() -> Result<(), anyhow::Error> {
    use serde_json::Value;
    use std::fs;

    // Clean up any existing instances.json file
    let _ = fs::remove_file("instances.json");

    let user = User::entity();
    let comment = Comment::entity();

    let db = Deeb::new();
    db.add_instance(
        "users",
        "./db/test_save_config_users.json",
        vec![user.clone()],
    )
    .await?;
    db.add_instance(
        "comments",
        "./db/test_save_config_comments.json",
        vec![comment.clone()],
    )
    .await?;

    // Save configuration to default path
    db.save_instance_config(None).await?;

    // Verify file was created
    assert!(fs::metadata("instances.json").is_ok());

    // Read and verify content
    let content = fs::read_to_string("instances.json")?;
    let config: Value = serde_json::from_str(&content)?;

    // Verify structure
    assert!(config.is_object());
    let config_obj = config.as_object().unwrap();

    // Should have both instances
    assert!(config_obj.contains_key("users"));
    assert!(config_obj.contains_key("comments"));

    // Verify users instance
    let users_instance = &config_obj["users"];
    assert!(users_instance.is_object());
    assert!(users_instance["entities"].is_array());

    let user_entities = users_instance["entities"].as_array().unwrap();
    assert_eq!(user_entities.len(), 1);
    assert_eq!(user_entities[0]["name"], "user");

    // Verify comments instance
    let comments_instance = &config_obj["comments"];
    assert!(comments_instance.is_object());
    assert!(comments_instance["entities"].is_array());

    let comment_entities = comments_instance["entities"].as_array().unwrap();
    assert_eq!(comment_entities.len(), 1);
    assert_eq!(comment_entities[0]["name"], "comment");

    // Clean up
    let _ = fs::remove_file("instances.json");
    let _ = fs::remove_file("./db/test_save_config_users.json");
    let _ = fs::remove_file("./db/test_save_config_comments.json");

    Ok(())
}

#[tokio::test]
async fn test_save_instance_config_custom_path() -> Result<(), anyhow::Error> {
    use serde_json::Value;
    use std::fs;

    let custom_path = "./db/custom_config.json";
    let _ = fs::remove_file(custom_path);

    let user = User::entity();

    let db = Deeb::new();
    db.add_instance(
        "test_instance",
        "./db/test_save_custom_path.json",
        vec![user.clone()],
    )
    .await?;

    // Save configuration to custom path
    db.save_instance_config(Some(custom_path)).await?;

    // Verify file was created at custom path
    assert!(fs::metadata(custom_path).is_ok());

    // Read and verify content
    let content = fs::read_to_string(custom_path)?;
    let config: Value = serde_json::from_str(&content)?;

    // Verify structure
    assert!(config.is_object());
    let config_obj = config.as_object().unwrap();
    assert!(config_obj.contains_key("test_instance"));

    let instance = &config_obj["test_instance"];
    assert!(instance["entities"].is_array());
    let entities = instance["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0]["name"], "user");

    // Clean up
    let _ = fs::remove_file(custom_path);
    let _ = fs::remove_file("./db/test_save_custom_path.json");

    Ok(())
}

#[tokio::test]
async fn test_save_instance_config_with_associations_and_indexes() -> Result<(), anyhow::Error> {
    use serde_json::Value;
    use std::fs;

    let config_path = "./db/config_with_features.json";
    let _ = fs::remove_file(config_path);

    // Create entities with associations and indexes
    let mut user = User::entity();
    user.add_index("age_index", vec!["age"], None)?;
    user.add_index("name_age_index", vec!["name", "age"], None)?;

    let comment = Comment::entity();

    let db = Deeb::new();
    db.add_instance(
        "main",
        "./db/test_features.json",
        vec![user.clone(), comment.clone()],
    )
    .await?;

    // Save configuration
    db.save_instance_config(Some(config_path)).await?;

    // Read and verify content
    let content = fs::read_to_string(config_path)?;
    let config: Value = serde_json::from_str(&content)?;

    let config_obj = config.as_object().unwrap();
    let main_instance = &config_obj["main"];
    let entities = main_instance["entities"].as_array().unwrap();

    // Find user entity in the array
    let user_entity = entities
        .iter()
        .find(|e| e["name"] == "user")
        .expect("User entity should exist");

    // Verify indexes are saved
    assert!(user_entity["indexes"].is_array());
    let indexes = user_entity["indexes"].as_array().unwrap();
    assert_eq!(indexes.len(), 2);

    // Verify primary key is saved
    assert_eq!(user_entity["primary_key"], "id");

    // Clean up
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file("./db/test_features.json");

    Ok(())
}

#[tokio::test]
async fn test_save_instance_config_empty_database() -> Result<(), anyhow::Error> {
    use serde_json::Value;
    use std::fs;

    let config_path = "./db/empty_config.json";
    let _ = fs::remove_file(config_path);

    let db = Deeb::new();

    // Save configuration with no instances
    db.save_instance_config(Some(config_path)).await?;

    // Verify file was created
    assert!(fs::metadata(config_path).is_ok());

    // Read and verify content
    let content = fs::read_to_string(config_path)?;
    let config: Value = serde_json::from_str(&content)?;

    // Should be empty object
    assert!(config.is_object());
    let config_obj = config.as_object().unwrap();
    assert!(config_obj.is_empty());

    // Clean up
    let _ = fs::remove_file(config_path);

    Ok(())
}

#[tokio::test]
async fn test_save_instance_config_excludes_data() -> Result<(), anyhow::Error> {
    use std::fs;

    let config_path = "./db/no_data_config.json";
    let _ = fs::remove_file(config_path);

    let user = User::entity();

    let db = Deeb::new();
    db.add_instance("users", "./db/test_no_data.json", vec![user.clone()])
        .await?;

    // Insert some data
    User::insert_one(
        &db,
        User {
            id: 1,
            name: "Test User".to_string(),
            age: 25.0,
        },
        None,
    )
    .await?;

    // Save configuration
    db.save_instance_config(Some(config_path)).await?;

    // Read and verify content
    let content = fs::read_to_string(config_path)?;

    // Verify no data is included in config
    let config_str = content.to_lowercase();
    println!("CONFIG STR: {}", config_str);
    assert!(!config_str.contains("test user"));
    assert!(!config_str.contains("data"));

    // But should contain entity configuration
    assert!(config_str.contains("entities"));
    assert!(config_str.contains("user"));

    // Clean up
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file("./db/test_no_data.json");

    Ok(())
}
