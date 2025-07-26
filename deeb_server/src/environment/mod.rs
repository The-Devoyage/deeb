use std::env;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Environment {
    pub jwt_secret: String,
}

impl Environment {
    pub fn new() -> Result<Self, dotenvy::Error> {
        dotenvy::dotenv()?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| dotenvy::Error::EnvVar(env::VarError::NotPresent))?;

        Ok(Environment { jwt_secret })
    }
}
