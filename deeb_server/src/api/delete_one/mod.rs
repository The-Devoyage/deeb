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
pub struct DeleteOnePayload {
    query: Option<Query>,
}

#[post("/delete-one/{entity_name}")]
pub async fn delete_one(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<DeleteOnePayload>,
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
        &AccessOperation::DeleteOne,
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

    let record = database
        .deeb
        .find_one::<Value>(&entity, query.clone(), None)
        .await;

    if record.is_err() {
        return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
            .message("Something went wrong when finding the record to delete.");
    }

    let record = record.unwrap();

    if record.is_none() {
        return Response::new(StatusCode::NOT_FOUND).message("Failed to find record to delete.");
    }

    let record = record.unwrap();

    let allowed = app_data.rules_worker.check_rules(
        &AccessOperation::DeleteOne,
        &path.entity_name,
        user.0,
        vec![record],
    );

    match allowed {
        Ok(allowed) => {
            if allowed {
                match database.deeb.delete_one(&entity, query, None).await {
                    Ok(Some(is_deleted)) => {
                        Response::new(StatusCode::OK).data(serde_json::Value::Bool(is_deleted))
                    }
                    Ok(None) => Response::new(StatusCode::OK).message("Document not found."),
                    Err(err) => {
                        log::error!("{:?}", err);
                        Response::new(StatusCode::INTERNAL_SERVER_ERROR).message(&err.to_string())
                    }
                }
            } else {
                return Response::new(StatusCode::FORBIDDEN)
                    .message("Access to delete this document denied.");
            }
        }
        Err(e) => {
            log::error!("{:?}", e);
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to check rules.");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{register_and_login_user, setup_test_app};
    use actix_web::{http::header, test};
    use serde_json::json;

    #[actix_web::test]
    async fn test_delete_one() {
        let app = test::init_service(setup_test_app(Some("test_delete_one")).await).await;
        let token = register_and_login_user(&app).await;

        let req = test::TestRequest::post()
            .uri("/insert-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(json!({"name": "Maple"}).to_string())
            .to_request();
        test::call_service(&app, req).await;

        let req = test::TestRequest::post()
            .uri("/delete-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(json!({"query": {"Eq": ["name", "Maple"]}}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
