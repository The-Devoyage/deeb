use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FindManyOptions {
    pub skip: Option<i32>,
    pub limit: Option<i32>,
}
