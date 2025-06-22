use actix_web::http::StatusCode;
use actix_web::{Responder, post, web};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use deeb::{Entity, Query};
use jsonwebtoken::encode;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::api::Response;
use crate::api::auth::register::User;
use crate::app_data::AppData;
use crate::auth::claims::Claims;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[post("/auth/login")]
async fn login(app_data: web::Data<AppData>, payload: web::Json<LoginRequest>) -> impl Responder {
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

    // 1. Look up user
    let user = match database
        .deeb
        .find_one::<User>(&entity, Query::eq("email", payload.email.clone()), None)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            log::error!("Failed to find user.");
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to find user.");
        }
        Err(err) => {
            log::error!("{:?}", err);
            return Response::new(StatusCode::UNAUTHORIZED).message("Invalid credentials.");
        }
    };

    // 2. Verify password
    let parsed_hash = PasswordHash::new(&user.password);
    let is_valid = parsed_hash
        .and_then(|hash| Argon2::default().verify_password(payload.password.as_bytes(), &hash))
        .is_ok();

    if !is_valid {
        log::error!("Invalid credentials.");
        return Response::new(StatusCode::UNAUTHORIZED).message("Invalid credentials.");
    }

    // 3. Create JWT
    let claims = Claims {
        sub: user._id.clone(), // or email
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        email: user.email,
    };

    let jwt_secret = &app_data.environment.jwt_secret;
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Token generation failed.");
        }
    };

    // 4. Return token
    Response::new(StatusCode::OK)
        .data(serde_json::to_value(LoginResponse { token }).unwrap())
        .message("Authenticated")
}

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_test_app;
    use actix_web::{http::header, test};
    use serde_json::json;

    #[actix_web::test]
    async fn test_login_user() {
        use actix_web::{http::header, test};
        use serde_json::json;

        // Setup test app
        let app = test::init_service(setup_test_app(Some("test_login_user")).await).await;

        // First, register the user
        let register_payload = json!({
            "email": "login_test@example.com",
            "password": "test1234",
            "name": "Login Tester"
        });

        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(register_payload.to_string())
            .to_request();

        let register_resp = test::call_service(&app, register_req).await;
        assert!(register_resp.status().is_success());

        // Now, login
        let login_payload = json!({
            "email": "login_test@example.com",
            "password": "test1234"
        });

        let login_req = test::TestRequest::post()
            .uri("/auth/login")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(login_payload.to_string())
            .to_request();

        let login_resp = test::call_service(&app, login_req).await;

        // Check success
        assert!(login_resp.status().is_success());

        // Check token exists in response
        let body = test::read_body(login_resp).await;
        let json_body: serde_json::Value = serde_json::from_slice(&body).expect("Invalid JSON");

        assert!(json_body["data"]["token"].is_string());
        assert_eq!(json_body["message"], "Authenticated");
    }

    #[actix_web::test]
    async fn test_login_user_invalid_password() {
        let app =
            test::init_service(setup_test_app(Some("test_login_user_invalid_password")).await)
                .await;

        // Register valid user
        let _ = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/auth/register")
                .insert_header((header::CONTENT_TYPE, "application/json"))
                .set_payload(
                    json!({
                        "email": "wrong_pass@example.com",
                        "password": "correctpass",
                        "name": "Wrong Pass"
                    })
                    .to_string(),
                )
                .to_request(),
        )
        .await;

        // Attempt login with wrong password
        let req = test::TestRequest::post()
            .uri("/auth/login")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(
                json!({
                    "email": "wrong_pass@example.com",
                    "password": "incorrectpass"
                })
                .to_string(),
            )
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }
}
