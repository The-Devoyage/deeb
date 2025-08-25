use actix_web::{
    Responder,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use deeb::{Entity, FindManyOptions, Query};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{DeebPath, Response};

use crate::{app_data::AppData, auth::auth_user::MaybeAuthUser, rules::AccessOperation};

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct FindManyPayload {
    query: Option<Query>,
    find_many_options: Option<FindManyOptions>,
}

#[post("/find-many/{entity_name}")]
pub async fn find_many(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<Option<FindManyPayload>>,
    user: MaybeAuthUser,
) -> impl Responder {
    let database = app_data.database.clone();
    let entity = Entity::new(&path.entity_name);

    // Create Instance
    match database
        .deeb
        .add_instance(
            format!("{}-{}", &path.entity_name, app_data.instance_name.as_str()).as_str(),
            &format!("./db/{}.json", app_data.instance_name),
            vec![entity.clone()],
        )
        .await
    {
        Ok(_) => {}
        Err(err) => {
            log::error!("{:?}", err);
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to get instance.");
        }
    };

    let applied_query = match app_data.rules_worker.get_query(
        &AccessOperation::FindMany,
        &path.entity_name,
        user.0.clone(),
        serde_json::to_value(payload.clone()).ok(),
    ) {
        Ok(q) => q,
        Err(err) => {
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR).message(&err.to_string());
        }
    };

    let client_query = match payload.clone().unwrap_or_default().query.clone() {
        Some(q) => q,
        None => Query::All,
    };

    // Combine client and applied queries
    let query = if !applied_query.is_null() {
        let jsonquery = serde_json::from_value::<Query>(applied_query);
        if jsonquery.is_err() {
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to get default query.");
        }
        Query::and(vec![client_query, jsonquery.unwrap()])
    } else {
        client_query
    };

    match database
        .deeb
        .find_many::<Value>(
            &entity,
            query,
            payload
                .clone()
                .unwrap_or_default()
                .find_many_options
                .clone(),
            None,
        )
        .await
    {
        Ok(Some(values)) => {
            let allowed = app_data.rules_worker.check_rules(
                &AccessOperation::FindMany,
                &path.entity_name,
                user.0,
                values.clone(),
            );
            match allowed {
                Ok(is_allowed) => {
                    if is_allowed {
                        let array = serde_json::Value::Array(values);
                        return Response::new(StatusCode::OK)
                            .data(array)
                            .message("Documents Found.");
                    } else {
                        log::error!("Access denied. Rule has prevented access to this resource.");
                        Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                            .message("Access denied. Rule has prevented access to this resource.")
                    }
                }
                Err(e) => {
                    log::error!("Access denied: {:?}", e);
                    Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                        .message("Access denied. Error while processing rules.")
                }
            }
        }
        Ok(None) => {
            log::warn!("NO DOCS FOUND");
            Response::new(StatusCode::OK).message("No documents found.")
        }
        Err(err) => {
            log::error!("{:?}", err);
            Response::new(StatusCode::INTERNAL_SERVER_ERROR).message(&err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::test_utils::{register_and_login_user, setup_test_app};
    use actix_web::{http::header, test};
    use serde_json::{Value, json};

    #[actix_web::test]
    async fn test_find_many() {
        let app = test::init_service(setup_test_app(Some("test_find_many")).await).await;
        let token = register_and_login_user(&app).await;

        let req = test::TestRequest::post()
            .uri("/insert-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(
                serde_json::Value::Array(vec![
                    json!({"name": "Scooter"}),
                    json!({"name": "Mango"}),
                    json!({"name": "Banjo"}),
                ])
                .to_string(),
            )
            .to_request();
        test::call_service(&app, req).await;

        let req = test::TestRequest::post()
            .uri("/find-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(json!({}).to_string())
            .to_request();

        let resp = test::call_service(&app, req).await;

        println!("RESP API: {resp:?}");

        assert!(resp.status().is_success());

        let body_bytes = test::read_body(resp).await;
        let result: Value =
            serde_json::from_slice(&body_bytes).expect("Response body should be valid JSON");

        let items = result
            .get("data")
            .and_then(|v| v.as_array())
            .expect("Expected 'data' field to be an array");

        assert_eq!(items.len(), 3, "Expected exactly 3 dog documents");

        let received_names: HashSet<&str> = items
            .iter()
            .map(|doc| doc.get("name").unwrap().as_str().unwrap())
            .collect();

        let expected_names: HashSet<&str> = ["Scooter", "Mango", "Banjo"].into_iter().collect();

        assert_eq!(received_names, expected_names, "Mismatch in dog names");
    }
}
