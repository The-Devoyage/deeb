use actix_web::{
    Responder,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use deeb::{Entity, Query};
use serde::Deserialize;
use serde_json::Value;

use super::{DeebPath, Response};

use crate::app_data::AppData;

#[derive(Deserialize)]
pub struct UpdateManyPayload {
    query: Option<Query>,
    document: Value,
}

#[post("/update-many/{entity_name}")]
pub async fn update_many(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<UpdateManyPayload>,
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
}

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_test_app;
    use actix_web::{http::header, test};
    use serde_json::json;

    use super::*;

    #[actix_web::test]
    async fn test_update_many() {
        let app = test::init_service(setup_test_app(Some("test_update_many")).await).await;

        let req = test::TestRequest::post()
            .uri("/insert-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
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
            .set_payload(json!({"query": json, "document": {"name": "Delbo"}}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
