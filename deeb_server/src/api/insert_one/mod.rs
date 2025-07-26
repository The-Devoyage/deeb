use actix_web::{
    Responder,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use deeb::Entity;
use serde_json::Value;

use super::Response;

use crate::{
    api::DeebPath, app_data::AppData, auth::auth_user::MaybeAuthUser, rules::AccessOperation,
};

#[post("/insert-one/{entity_name}")]
pub async fn insert_one(
    app_data: Data<AppData>,
    mut document: Json<Value>,
    path: Path<DeebPath>,
    user: MaybeAuthUser,
) -> impl Responder {
    let database = app_data.database.clone();
    let entity = Entity::new(&path.entity_name);

    if let Some(user) = user.0.clone() {
        if let Some(doc_obj) = document.as_object_mut() {
            doc_obj.insert(
                "_created_by".to_string(),
                Value::String(user._id.to_string()),
            );
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

    let allowed = app_data.rules_worker.check_rules(
        &AccessOperation::InsertOne,
        &path.entity_name,
        user.0,
        vec![],
    );

    if allowed.is_err() {
        return Response::new(StatusCode::INTERNAL_SERVER_ERROR).message("Failed to check insert rules.");
    }

    if !allowed.unwrap() {
        return Response::new(StatusCode::FORBIDDEN).message("Insert access denied.");
    }

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
    use actix_web::{http::header, test};
    use serde_json::json;

    use crate::test_utils::{register_and_login_user, setup_test_app};

    #[actix_web::test]
    async fn test_insert_one() {
        let app = test::init_service(setup_test_app(Some("test_insert_one")).await).await;
        let token = register_and_login_user(&app).await;

        let req = test::TestRequest::post()
            .uri("/insert-one/dog")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token.0)))
            .set_payload(json!({"name": "Bongo"}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
