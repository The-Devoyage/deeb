use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OrderDirection {
    Ascending,
    Descending,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FindManyOrder {
    pub property: String,
    pub direction: OrderDirection,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FindManyOptions {
    pub skip: Option<i32>,
    pub limit: Option<i32>,
    pub order: Option<Vec<FindManyOrder>>,
}
