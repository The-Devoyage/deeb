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
pub struct DeleteManyPayload {
    query: Option<Query>,
}

#[post("/delete-many/{entity_name}")]
pub async fn delete_many(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<DeleteManyPayload>,
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
        &AccessOperation::DeleteMany,
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
            .message("Something went wrong when finding documents to delete.");
    }

    let records = records.unwrap();

    if records.is_none() {
        return Response::new(StatusCode::NOT_FOUND).message("Failed to find documents to delete.");
    }

    let records = records.unwrap();

    let allowed = app_data.rules_worker.check_rules(
        &AccessOperation::DeleteMany,
        &path.entity_name,
        user.0,
        records,
    );

    match allowed {
        Ok(allowed) => {
            if allowed {
                match database.deeb.delete_many(&entity, query, None).await {
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
                    .message("Access to delete these records denied.");
            }
        }
        Err(e) => {
            log::error!("{:?}", e);
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Something went wrong when checking delete many rules.");
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{http::header, test};
    use serde_json::json;

    use crate::test_utils::{register_and_login_user, setup_test_app};

    #[actix_web::test]
    async fn test_delete_many() {
        let app = test::init_service(setup_test_app(Some("test_delete_many")).await).await;
        let token = register_and_login_user(&app).await;

        let req = test::TestRequest::post()
            .uri("/insert-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(
                serde_json::Value::Array(vec![
                    json!({"name": "bizz"}),
                    json!({"name": "bazz"}),
                    json!({"name": "buzz"}),
                ])
                .to_string(),
            )
            .to_request();
        test::call_service(&app, req).await;

        let req = test::TestRequest::post()
            .uri("/delete-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(json!({"query": {"Like": ["name", "zz"]}}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
