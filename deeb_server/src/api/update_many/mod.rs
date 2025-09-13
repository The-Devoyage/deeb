use actix_web::{
    Responder,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use deeb::{Entity, Query};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{DeebPath, Response};

use crate::{app_data::AppData, auth::auth_user::MaybeAuthUser, rules::AccessOperation};

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateManyPayload {
    query: Option<Query>,
    document: Value,
}

#[post("/update-many/{entity_name}")]
pub async fn update_many(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<UpdateManyPayload>,
    user: MaybeAuthUser,
) -> impl Responder {
    let database = app_data.database.clone();
    let entity = Entity::new(&path.entity_name);

    let applied_query = match app_data.rules_worker.get_query(
        &AccessOperation::UpdateMany,
        &path.entity_name,
        user.0.clone(),
        serde_json::to_value(payload.clone()).ok(),
    ) {
        Ok(q) => q,
        Err(err) => {
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR).message(&err.to_string());
        }
    };

    let client_query = match payload.query.clone() {
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

    // Check rules
    let records = database
        .deeb
        .find_many::<Value>(&entity, query.clone(), None, None)
        .await;

    if records.is_err() {
        let _ = records.inspect_err(|e| log::error!("{:?}", e));
        return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
            .message("Something went wrong when finding documents to modify.");
    }

    let records = records.unwrap();

    if records.is_none() {
        return Response::new(StatusCode::NOT_FOUND).message("Failed to find documents to modify.");
    }

    let records = records.unwrap();

    let allowed = app_data.rules_worker.check_rules(
        &AccessOperation::UpdateMany,
        &path.entity_name,
        user.0,
        records,
    );

    match allowed {
        Ok(allowed) => {
            if allowed {
                match database
                    .deeb
                    .update_many::<Value, Value>(&entity, query, payload.document.clone(), None)
                    .await
                {
                    Ok(Some(values)) => {
                        let json = serde_json::Value::Array(values);
                        Response::new(StatusCode::OK).data(json)
                    }
                    Ok(None) => Response::new(StatusCode::OK).message("Document not found."),
                    Err(err) => {
                        log::error!("{:?}", err);
                        Response::new(StatusCode::INTERNAL_SERVER_ERROR).message(&err.to_string())
                    }
                }
            } else {
                return Response::new(StatusCode::FORBIDDEN).message("Access to resource denied.");
            }
        }
        Err(e) => {
            log::error!("{:?}", e);
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Something went wrong when checking rules");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{register_and_login_user, setup_test_app};
    use actix_web::{http::header, test};
    use serde_json::json;

    use super::*;

    #[actix_web::test]
    async fn test_update_many() {
        let app = test::init_service(setup_test_app(Some("test_update_many")).await).await;
        let token = register_and_login_user(&app).await;

        let req = test::TestRequest::post()
            .uri("/insert-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(
                serde_json::Value::Array(vec![
                    json!({"name": "Cheyenne"}),
                    json!({"name": "Sparky"}),
                    json!({"name": "Missy"}),
                ])
                .to_string(),
            )
            .to_request();
        test::call_service(&app, req).await;

        let payload = Query::or(vec![
            Query::eq("name", "Cheyenne"),
            Query::eq("name", "Sparky"),
            Query::eq("name", "Missy"),
        ]);

        let json = serde_json::to_value(payload.clone()).unwrap();

        let req = test::TestRequest::post()
            .uri("/update-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(json!({"query": json, "document": {"name": "Delbo"}}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
