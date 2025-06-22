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
pub struct DeleteOnePayload {
    query: Option<Query>,
}

#[post("/delete-one/{entity_name}")]
pub async fn delete_one(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<DeleteOnePayload>,
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
}

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_test_app;
    use actix_web::{http::header, test};
    use serde_json::json;

    #[actix_web::test]
    async fn test_delete_one() {
        let app = test::init_service(setup_test_app(Some("test_delete_one")).await).await;

        let req = test::TestRequest::post()
            .uri("/insert-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(json!({"name": "Maple"}).to_string())
            .to_request();
        test::call_service(&app, req).await;

        let req = test::TestRequest::post()
            .uri("/delete-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(json!({"query": {"Eq": ["name", "Maple"]}}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
