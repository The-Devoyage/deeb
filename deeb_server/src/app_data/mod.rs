use super::database::Database;

#[derive(Clone)]
pub struct AppData {
    pub database: Database,
}
