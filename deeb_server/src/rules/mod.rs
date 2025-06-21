use core::fmt;
use serde_json::Value;
use std::{fmt::Display, sync::mpsc};
use thiserror::Error;

use crate::auth::auth_user::AuthUser;

pub mod create_rules;
pub mod load_rules;
pub mod worker;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("{0}")]
    ApplyQueryError(String),

    #[error("Worker failed to receive an appropriate response: {0}")]
    WorkerReceiveError(String),
}

pub struct ApplyQueryRequest {
    pub entity: String,
    pub operation: String,
    pub response_tx: mpsc::Sender<Result<Value, ScriptError>>,
    pub user: Option<AuthUser>,
    pub payload: Option<Value>,
}

pub struct CheckRuleRequest {
    pub entity: String,
    pub operation: String,
    pub resource: Value,
    pub user: Option<AuthUser>,
    pub response_tx: mpsc::Sender<Result<bool, ScriptError>>,
}

pub enum RhaiTask {
    ApplyQuery(ApplyQueryRequest),
    CheckRule(CheckRuleRequest),
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

#[derive(Clone)]
pub struct Rules {
    pub sender: mpsc::Sender<RhaiTask>,
}

impl Rules {
    pub fn check_rules(
        &self,
        operation: &AccessOperation,
        entity: &str,
        user: Option<AuthUser>,
        values: Vec<Value>,
    ) -> Result<bool, ScriptError> {
        for doc in values {
            let (response_tx, response_rx) = mpsc::channel();

            let req = CheckRuleRequest {
                entity: entity.to_string(),
                operation: operation.to_string(),
                resource: doc.clone(),
                user,
                response_tx,
            };

            let task = RhaiTask::CheckRule(req);

            if let Err(e) = self.sender.send(task) {
                log::error!("Failed to send CheckRule task to Rhai worker: {:?}", e);
                return Ok(false);
            }

            match response_rx.recv() {
                Ok(allowed) => return allowed,
                Err(e) => {
                    log::error!("Failed to receive Rhai result: {:?}", e);
                    return Ok(false);
                }
            };
        }

        Ok(true)
    }

    pub fn get_query(
        &self,
        operation: &AccessOperation,
        entity: &str,
        user: Option<AuthUser>,
        payload: Option<Value>,
    ) -> Result<Value, ScriptError> {
        let (response_tx, response_rx) = mpsc::channel();

        let req = ApplyQueryRequest {
            entity: entity.to_string(),
            operation: operation.to_string(),
            response_tx,
            user,
            payload,
        };

        let task = RhaiTask::ApplyQuery(req);

        if let Err(e) = self.sender.send(task) {
            log::error!("Failed to send ApplyQuery task to Rhai worker: {:?}", e);
        }

        match response_rx.recv() {
            Ok(value) => value,
            Err(e) => {
                log::error!("Failed to receive Rhai ApplyQuery result: {:?}", e);
                Err(ScriptError::WorkerReceiveError(
                    "Failed to receive apply query result.".to_string(),
                ))
            }
        }
    }
}
