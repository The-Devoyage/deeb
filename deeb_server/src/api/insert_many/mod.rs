use actix_web::{
    Responder,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use deeb::Entity;
use serde_json::Value;

use super::Response;

use crate::{api::DeebPath, app_data::AppData, auth::auth_user::MaybeAuthUser};

#[post("/insert-many/{entity_name}")]
pub async fn insert_many(
    app_data: Data<AppData>,
    mut document: Json<Vec<Value>>,
    path: Path<DeebPath>,
    user: MaybeAuthUser,
) -> impl Responder {
    let database = app_data.database.clone();
    let entity = Entity::new(&path.entity_name);

    // If user is authenticated, add _created_by to each document
    if let Some(user) = user.0 {
        for doc in document.iter_mut() {
            if let Some(obj) = doc.as_object_mut() {
                obj.insert(
                    "_created_by".to_string(),
                    Value::String(user._id.to_string()),
                );
            }
        }
    }

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

    // Insert Payload
    match database
        .deeb
        .insert_many(&entity, document.into_inner(), None)
        .await
    {
        Ok(values) => {
            let json_array = serde_json::Value::Array(values);
            Response::new(StatusCode::OK)
                .data(json_array)
                .message("Documents inserted.")
        }
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
    async fn test_insert_many() {
        let app = test::init_service(setup_test_app(Some("test_insert_many")).await).await;

        let req = test::TestRequest::post()
            .uri("/insert-many/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(
                serde_json::Value::Array(vec![
                    json!({"name": "bozo"}),
                    json!({"name": "bingo"}),
                    json!({"name": "congo"}),
                ])
                .to_string(),
            )
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
