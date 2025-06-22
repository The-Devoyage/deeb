use actix_web::{
    Responder,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use deeb::{Entity, Query};
use serde::Deserialize;

use super::{DeebPath, Response};

use crate::app_data::AppData;

#[derive(Deserialize)]
pub struct DeleteManyPayload {
    query: Option<Query>,
}

#[post("/delete-many/{entity_name}")]
pub async fn delete_many(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<DeleteManyPayload>,
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

    let query = match payload.query.clone() {
        Some(q) => q,
        None => Query::All,
    };

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
}

#[cfg(test)]
mod tests {
    use actix_web::{http::header, test};
    use serde_json::json;

    use crate::test_utils::setup_test_app;

    #[actix_web::test]
    async fn test_delete_many() {
        let app = test::init_service(setup_test_app(Some("test_delete_many")).await).await;

        let req = test::TestRequest::post()
            .uri("/insert-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
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
            .set_payload(json!({"query": {"Like": ["name", "zz"]}}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
