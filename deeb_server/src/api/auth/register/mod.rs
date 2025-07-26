use actix_web::http::StatusCode;
use actix_web::{Responder, post, web};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use deeb::{Entity, Query};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::api::Response;
use crate::app_data::AppData;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub _id: String,
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[post("/auth/register")]
async fn register_user(
    data: web::Json<RegisterRequest>,
    app_data: web::Data<AppData>,
) -> impl Responder {
    let req = data.into_inner();

    let database = app_data.database.clone();
    let entity = Entity::new(&"user");
    match database
        .deeb
        .add_instance(
            format!("{}-{}", &"user", app_data.instance_name.as_str()).as_str(),
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

    // Check if user already exists
    match database
        .deeb
        .find_one::<User>(&entity, Query::eq("email", req.email.clone()), None)
        .await
    {
        Ok(user) => {
            if user.is_some() {
                return Response::new(StatusCode::CONFLICT).message("User already exists.");
            }
        }
        Err(err) => {
            log::error!("{:?}", err);
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to check for existing users.");
        }
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(req.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to hash password.");
        }
    };

    // Save user
    match database
        .deeb
        .insert_one::<CreateUser, serde_json::Value>(
            &entity,
            CreateUser {
                email: req.email,
                password: password_hash,
                name: req.name,
            },
            None,
        )
        .await
    {
        Ok(_) => {},
        Err(err) => {
            log::error!("{:?}", err);
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to insert user.");
        }
    };

    Response::new(StatusCode::OK).message("Successfully Registered")
}

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_test_app;
    use actix_web::{
        http::{StatusCode, header},
        test,
    };
    use serde_json::json;

    #[actix_web::test]
    async fn test_register_duplicate_user() {
        let app =
            test::init_service(setup_test_app(Some("test_register_duplicate_user")).await).await;

        let payload = json!({
            "email": "dup@example.com",
            "password": "somepass",
            "name": "Duplicate"
        });

        // First registration (should succeed)
        let req = test::TestRequest::post()
            .uri("/auth/register")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(payload.to_string())
            .to_request();
        let _ = test::call_service(&app, req).await;

        // Second registration (should fail)
        let req = test::TestRequest::post()
            .uri("/auth/register")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(payload.to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CONFLICT);

        let body = test::read_body(resp).await;
        let json_body: serde_json::Value =
            serde_json::from_slice(&body).expect("Invalid JSON response");

        assert_eq!(json_body["message"], "User already exists.");
    }
}
