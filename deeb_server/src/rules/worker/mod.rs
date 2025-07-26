use rhai::serde::from_dynamic;
use rhai::{Array, Dynamic, Engine, Map, Scope};
use serde_json::Value;
use std::sync::mpsc;
use std::thread;

use crate::rules::ScriptError;

use super::{RhaiTask, Rules};

impl Rules {
    pub fn new(script: String) -> Self {
        let (tx, rx) = mpsc::channel::<RhaiTask>();

        thread::spawn(move || {
            let mut engine = Engine::new();
            engine.set_max_expr_depths(64, 64);

            let ast = engine.compile(&script).expect("Compile failed");

            for task in rx {
                let mut scope = Scope::new();

                match task {
                    RhaiTask::ApplyQuery(req) => {
                        let result = engine.call_fn::<Dynamic>(
                            &mut scope,
                            &ast,
                            "apply_query",
                            (
                                req.entity,
                                req.operation,
                                Rules::json_value_to_dynamic(
                                    &serde_json::to_value(req.user).unwrap(),
                                ),
                                Rules::json_value_to_dynamic(&req.payload.unwrap_or_default()),
                            ),
                        );
                        let json_result = match result {
                            Ok(dynamic) => Ok(from_dynamic(&dynamic).unwrap_or(Value::Null)),
                            Err(e) => {
                                log::error!("Apply Query Error: {:?}", e.to_string());
                                let error = Self::clean_rhai_error(&e);
                                Err(ScriptError::ApplyQueryError(error))
                            }
                        };

                        let _ = req.response_tx.send(json_result);
                    }
                    RhaiTask::CheckRule(req) => {
                        let resource = Self::json_value_to_dynamic(&req.resource);
                        let result = engine.call_fn::<bool>(
                            &mut scope,
                            &ast,
                            "check_rule",
                            (
                                req.entity,
                                req.operation,
                                Rules::json_value_to_dynamic(
                                    &serde_json::to_value(req.user).unwrap(),
                                ),
                                resource,
                            ),
                        );
                        let _ = req.response_tx.send(result.map_err(|e| {
                            ScriptError::CheckRuleError(e.to_string())
                        }));
                    }
                }
            }
        });

        Rules { sender: tx }
    }

    pub fn clean_rhai_error(err: &rhai::EvalAltResult) -> String {
        let full_msg = err.to_string();

        // Strip "Runtime error: " prefix if present
        let trimmed = full_msg
            .strip_prefix("Runtime error: ")
            .unwrap_or(&full_msg);

        // Remove position info if present (e.g. " (line 12, position 9)")
        let cleaned = trimmed
            .rsplit_once(" (line ")
            .map(|(msg, _)| msg.trim())
            .unwrap_or(trimmed.trim());

        cleaned.to_string()
    }

    pub fn json_value_to_dynamic(value: &serde_json::Value) -> Dynamic {
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
                let rhai_array: Array = arr.iter().map(Self::json_value_to_dynamic).collect();
                Dynamic::from_array(rhai_array)
            }
            serde_json::Value::Object(obj) => {
                let mut map = Map::new();
                for (k, v) in obj {
                    map.insert(k.clone().into(), Self::json_value_to_dynamic(v));
                }
                Dynamic::from_map(map)
            }
        }
    }
}
