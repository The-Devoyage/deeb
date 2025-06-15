use actix_web::{HttpResponse, Responder, get};

use crate::auth::auth_user::AuthUser;

#[get("/me")]
pub async fn me(user: AuthUser) -> impl Responder {
    HttpResponse::Ok().json(user)
}
