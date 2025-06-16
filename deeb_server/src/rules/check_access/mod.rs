use core::fmt;
use std::fmt::Display;

use actix_web::http::StatusCode;
use rhai::{Engine, Scope};
use serde_json::{Value, ser::Formatter};

use crate::api::Response;

use rhai::{Array, Dynamic, Map as RhaiMap};

fn json_value_to_dynamic(value: &serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from_bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from_int(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from_float(f)
            } else {
                Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s.clone()),
        serde_json::Value::Array(arr) => {
            let rhai_array: Array = arr.iter().map(json_value_to_dynamic).collect();
            Dynamic::from_array(rhai_array)
        }
        serde_json::Value::Object(obj) => {
            let mut map = RhaiMap::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), json_value_to_dynamic(v));
            }
            Dynamic::from_map(map)
        }
    }
}

pub enum AccessOperation {
    FindOne,
    FindMany,
    InsertOne,
    InsertMany,
    UpdateOne,
    UpdateMany,
    DeleteOne,
    DeleteMany,
}

impl Display for AccessOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessOperation::FindOne => write!(f, "find_one"),
            AccessOperation::FindMany => write!(f, "find_many"),
            AccessOperation::InsertOne => write!(f, "insert_one"),
            AccessOperation::InsertMany => write!(f, "insert_many"),
            AccessOperation::UpdateOne => write!(f, "update_one"),
            AccessOperation::UpdateMany => write!(f, "update_many"),
            AccessOperation::DeleteOne => write!(f, "delete_one"),
            AccessOperation::DeleteMany => write!(f, "delete_many"),
        }
    }
}

pub fn check_access(
    rules: &str,
    operation: &AccessOperation,
    entity: &str,
    values: Vec<Value>,
) -> Response {
    let engine = Engine::new();

    // Compile once outside the loop
    let ast = match engine.compile(rules) {
        Ok(ast) => ast,
        Err(e) => {
            log::error!("Failed to compile rules: {:?}", e);
            return Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .message("Failed to compile rules.");
        }
    };

    let mut filtered_docs = Vec::new();

    for doc in values {
        let resource = json_value_to_dynamic(&doc);

        let mut scope = Scope::new();

        // Call can_access
        let allowed = match engine.call_fn::<bool>(
            &mut scope,
            &ast,
            "can_access",
            (
                entity.to_string(),
                operation.to_string(),
                RhaiMap::new(),
                resource,
            ),
        ) {
            Ok(val) => val,
            Err(err) => {
                log::error!("Rule evaluation error: {:?}", err);
                false
            }
        };

        if allowed {
            filtered_docs.push(doc);
        }
    }

    // Now build the JSON response from filtered docs
    let json_array = serde_json::Value::Array(filtered_docs);

    match operation {
        AccessOperation::FindMany | AccessOperation::InsertMany | AccessOperation::UpdateMany => {
            Response::new(StatusCode::OK).data(json_array)
        }
        //TODO: Return One
        _ => Response::new(StatusCode::OK).data(json_array),
    }
}
