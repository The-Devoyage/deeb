use actix_web::http::StatusCode;
use actix_web::{Responder, post, web};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Utc, Duration};
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

    // 1. Look up user
    let user = match database
        .deeb
        .find_one::<User>(&entity, Query::eq("email", payload.email.clone()), None)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
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
