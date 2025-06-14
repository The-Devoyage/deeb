use actix_web::{HttpRequest, HttpResponse, Responder, body::BoxBody, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod find_one;
pub mod insert_one;
pub mod find_many;
pub mod insert_many;
pub mod delete_one;
pub mod delete_many;

#[derive(Serialize)]
pub struct Response {
    #[serde(skip_serializing)]
    pub status_code: StatusCode,
    pub data: Option<Value>,
    pub message: Option<String>,
}

impl Response {
    pub fn new(status_code: StatusCode) -> Self {
        Response {
            status_code,
            data: None,
            message: None,
        }
    }

    pub fn data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }
}

impl Responder for Response {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::build(self.status_code)
            .content_type("application/json")
            .json(self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeebPath {
    entity_name: String,
}
