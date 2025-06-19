use actix_web::{Error, FromRequest, HttpRequest, dev::Payload, web::Data};
use futures_util::future::{Ready, err, ok, ready};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Serialize;

use crate::app_data::AppData;

use super::claims::Claims;

#[derive(Clone, Debug, Serialize)]
pub struct AuthUser {
    pub _id: String,
    pub email: String,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        AuthUser {
            _id: claims.sub,
            email: claims.email,
        }
    }
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let app_data = match req.app_data::<Data<AppData>>() {
            Some(data) => data,
            None => {
                return ready(Err(actix_web::error::ErrorInternalServerError(
                    "Missing app data",
                )));
            }
        };
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        if let Some(token) = auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
            let key = DecodingKey::from_secret(app_data.environment.jwt_secret.as_ref());
            let validation = Validation::default();

            match decode::<Claims>(token, &key, &validation) {
                Ok(data) => ok(data.claims.into()),
                Err(_) => err(actix_web::error::ErrorUnauthorized("Invalid token")),
            }
        } else {
            err(actix_web::error::ErrorUnauthorized("No auth header"))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequest for MaybeAuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        match AuthUser::from_request(req, payload).into_inner() {
            Ok(user) => ready(Ok(MaybeAuthUser(Some(user)))),
            Err(_) => ready(Ok(MaybeAuthUser(None))),
        }
    }
}
