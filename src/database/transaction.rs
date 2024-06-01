use uuid::Uuid;

use super::Operation;

pub struct Transaction {
    pub id: Uuid,
    pub operations: Vec<Operation>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            operations: Vec::new(),
        }
    }

    pub fn add_operation(&mut self, operation: Operation) -> &mut Self {
        self.operations.push(operation);
        self
    }
}
