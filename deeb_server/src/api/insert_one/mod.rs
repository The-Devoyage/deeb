use actix_web::{
    Responder,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use deeb::Entity;
use serde_json::Value;

use super::Response;

use crate::{api::DeebPath, app_data::AppData};

#[post("/insert-one/{entity_name}")]
pub async fn insert_one(
    app_data: Data<AppData>,
    document: Json<Value>,
    path: Path<DeebPath>,
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

    // Insert Payload
    match database
        .deeb
        .insert_one(&entity, document.into_inner(), None)
        .await
    {
        Ok(value) => Response::new(StatusCode::OK)
            .data(value)
            .message("Document inserted."),
        Err(err) => {
            log::error!("{:?}", err);
            Response::new(StatusCode::INTERNAL_SERVER_ERROR).message(&err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{App, http::header, test};
    use serde_json::json;

    use super::*;

    #[actix_web::test]
    async fn test_insert_one() {
        let app_data = AppData::new().unwrap();
        let app =
            test::init_service(App::new().app_data(Data::new(app_data)).service(insert_one)).await;
        let req = test::TestRequest::post()
            .uri("/insert-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(json!({"name": "Bongo"}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp.response());
        assert!(resp.status().is_success());
    }
}
