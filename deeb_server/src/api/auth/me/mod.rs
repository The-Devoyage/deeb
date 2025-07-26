use actix_web::{HttpResponse, Responder, get};

use crate::auth::auth_user::AuthUser;

#[get("/me")]
pub async fn me(user: AuthUser) -> impl Responder {
    HttpResponse::Ok().json(user)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_test_app;

    #[actix_web::test]
    async fn test_me_authenticated() {
        use actix_web::{http::header, test};
        use serde_json::Value;

        let app = test::init_service(setup_test_app(Some("test_me_authenticated")).await).await;

        // First, register and login to get a valid token
        let register_payload = serde_json::json!({
            "email": "me_test@example.com",
            "password": "test1234",
            "name": "Me Tester"
        });

        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(register_payload.to_string())
            .to_request();

        let register_resp = test::call_service(&app, register_req).await;
        assert!(register_resp.status().is_success());

        let login_payload = serde_json::json!({
            "email": "me_test@example.com",
            "password": "test1234"
        });

        let login_req = test::TestRequest::post()
            .uri("/auth/login")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(login_payload.to_string())
            .to_request();

        let login_resp = test::call_service(&app, login_req).await;
        assert!(login_resp.status().is_success());

        let login_body = test::read_body(login_resp).await;
        let login_json: Value = serde_json::from_slice(&login_body).unwrap();
        let token = login_json["data"]["token"].as_str().unwrap();

        // Call /me with Authorization header
        let me_req = test::TestRequest::get()
            .uri("/me")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let me_resp = test::call_service(&app, me_req).await;
        assert!(me_resp.status().is_success());

        let body = test::read_body(me_resp).await;
        let user_json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(user_json["email"], "me_test@example.com");
    }
}
