use actix_web::{
    Responder,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use deeb::{Entity, FindManyOptions, Query};
use serde::Deserialize;
use serde_json::Value;

use super::{DeebPath, Response};

use crate::{app_data::AppData, rules::check_access::{check_access, AccessOperation}};

#[derive(Deserialize)]
pub struct FindManyPayload {
    query: Option<Query>,
    find_many_options: Option<FindManyOptions>,
}

#[post("/find-many/{entity_name}")]
pub async fn find_many(
    app_data: Data<AppData>,
    path: Path<DeebPath>,
    payload: Option<Json<FindManyPayload>>,
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

    let payload = payload.unwrap_or(Json(FindManyPayload {
        query: None,
        find_many_options: None,
    }));

    let query = match payload.query.clone() {
        Some(q) => q,
        None => Query::All,
    };

    match database
        .deeb
        .find_many::<Value>(&entity, query, payload.find_many_options.clone(), None)
        .await
    {
        Ok(Some(values)) => check_access(
            &app_data.rules,
            &AccessOperation::FindMany,
            &path.entity_name,
            values,
        ),
        Ok(None) => Response::new(StatusCode::OK).message("No documents found."),
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
    async fn test_find_many() {
        let app_data = AppData::new(Some("./rules.rhai".to_string())).unwrap();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(app_data))
                .service(find_many)
                .service(insert_one),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/insert-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
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
            .to_request();
        let resp = test::call_service(&app, req).await;

        println!("{:?}", resp.response());

        assert!(resp.status().is_success());
    }
}
