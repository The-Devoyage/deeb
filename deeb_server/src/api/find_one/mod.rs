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
pub struct FindOnePayload {
    query: Option<Query>,
}

#[post("/find-one/{entity_name}")]
pub async fn find_one(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<FindOnePayload>,
    user: MaybeAuthUser,
) -> impl Responder {
    let database = app_data.database.clone();
    let entity = Entity::new(&path.entity_name);

    let applied_query = match app_data.rules_worker.get_query(
        &AccessOperation::FindOne,
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

    match database.deeb.find_one::<Value>(&entity, query, None).await {
        Ok(Some(value)) => {
            let allowed = app_data.rules_worker.check_rules(
                &AccessOperation::FindOne,
                &path.entity_name,
                user.0,
                vec![value.clone()],
            );
            match allowed {
                Ok(is_allowed) => {
                    if is_allowed {
                        return Response::new(StatusCode::OK)
                            .data(value)
                            .message("Document Found.");
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
        Ok(None) => Response::new(StatusCode::OK).message("Document not found."),
        Err(err) => {
            log::error!("{:?}", err);
            Response::new(StatusCode::INTERNAL_SERVER_ERROR).message(&err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{register_and_login_user, setup_test_app};
    use actix_web::{http::header, test};
    use serde_json::json;

    #[actix_web::test]
    async fn test_find_one() {
        let app = test::init_service(setup_test_app(Some("test_find_one")).await).await;
        let token = register_and_login_user(&app).await;

        let req = test::TestRequest::post()
            .uri("/insert-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(json!({"name": "Oakley"}).to_string())
            .to_request();
        test::call_service(&app, req).await;

        let req = test::TestRequest::post()
            .uri("/find-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(json!({"query": {"Eq": ["name", "Oakley"]}}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
