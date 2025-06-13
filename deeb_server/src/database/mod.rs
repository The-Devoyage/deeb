use deeb::Deeb;

#[derive(Clone)]
pub struct Database {
    pub deeb: Deeb,
}

impl Database {
    pub fn new() -> Self {
        let deeb = Deeb::new();

        Database { deeb }
    }
}
