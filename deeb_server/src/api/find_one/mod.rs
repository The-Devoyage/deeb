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
pub struct FindOnePayload {
    query: Option<Query>,
}

#[post("/find-one/{entity_name}")]
pub async fn find_one(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Json<FindOnePayload>,
) -> impl Responder {
    let database = app_data.database.clone();
    let entity = Entity::new(&path.entity_name);

    // Create Instance
    match database
        .deeb
        .add_instance(
            "instance_name",
            "./first_instance.json",
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

    match database.deeb.find_one::<Value>(&entity, query, None).await {
        Ok(Some(value)) => Response::new(StatusCode::OK).data(value),
        Ok(None) => Response::new(StatusCode::OK).message("Document not found."),
        Err(err) => {
            log::error!("{:?}", err);
            Response::new(StatusCode::INTERNAL_SERVER_ERROR).message(&err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::insert_one::insert_one;
    use actix_web::{App, http::header, test};
    use serde_json::json;

    use super::*;

    #[actix_web::test]
    async fn test_find_one() {
        let app_data = AppData::new(None).unwrap();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(app_data))
                .service(find_one)
                .service(insert_one),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/insert-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(json!({"name": "Oakley"}).to_string())
            .to_request();
        test::call_service(&app, req).await;

        let req = test::TestRequest::post()
            .uri("/find-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(json!({"query": {"Eq": ["name", "Oakley"]}}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        println!("{:?}", resp.response());

        assert!(resp.status().is_success());
    }
}
