use ulid::Ulid;

use super::Operation;

pub struct Transaction {
    pub id: Ulid,
    pub operations: Vec<Operation>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            id: Ulid::new(),
            operations: Vec::new(),
        }
    }

    pub fn add_operation(&mut self, operation: Operation) -> &mut Self {
        self.operations.push(operation);
        self
    }
}
