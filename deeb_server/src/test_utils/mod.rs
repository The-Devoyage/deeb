use std::fs;
use std::sync::Once;
use std::thread::sleep;

use actix_web::{App, web::Data};

use crate::api::{
    auth, delete_many, delete_one, find_many, find_one, insert_many, insert_one, update_many,
    update_one,
};
use crate::app_data::AppData;
use actix_web::{http::header, test};
use serde_json::Value;
use serde_json::json;

static INIT: Once = Once::new();

pub async fn setup_test_app(
    instance_name: Option<&str>,
) -> actix_web::App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    INIT.call_once(|| {
        pretty_env_logger::init();
        let _ = fs::remove_dir("./db");
        sleep(std::time::Duration::from_secs(3));
        let _ = fs::create_dir("./db");
        log::info!("🧹 Test DB deleted before tests");
        sleep(std::time::Duration::from_secs(3));
    });

    let app_data = AppData::new(
        Some("./example-rules.rhai".to_string()),
        instance_name.map(|s| s.to_string()),
    )
    .expect("Failed to load app data");

    App::new()
        .app_data(Data::new(app_data))
        .service(update_one::update_one)
        .service(insert_one::insert_one)
        .service(find_one::find_one)
        .service(find_many::find_many)
        .service(insert_many::insert_many)
        .service(delete_one::delete_one)
        .service(delete_many::delete_many)
        .service(update_many::update_many)
        .service(auth::me::me)
        .service(auth::register::register_user)
        .service(auth::login::login)
}

pub async fn register_and_login_user(email: &str, password: &str) -> (String, String) {
    let app = test::init_service(setup_test_app(None).await).await;

    // 1. Register the user
    let register_payload = json!({
        "email": email,
        "password": password,
        "name": "Test User"
    });

    let register_req = test::TestRequest::post()
        .uri("/auth/register")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(register_payload.to_string())
        .to_request();

    let _ = test::call_service(&app, register_req).await;

    // 2. Log in
    let login_payload = json!({
        "email": email,
        "password": password,
    });

    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(login_payload.to_string())
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert!(login_resp.status().is_success());

    let body = test::read_body(login_resp).await;
    let json: Value = serde_json::from_slice(&body).expect("Invalid JSON");

    let token = json["data"]["token"]
        .as_str()
        .expect("Missing token")
        .to_string();

    let user_id = json["data"]["_id"]
        .as_str()
        .expect("Missing user ID")
        .to_string();

    (token, user_id)
}
