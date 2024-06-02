use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Key(String);

impl Key {}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    Eq(Key, Value),
    Ne(Key, Value),
    Like(Key, String),
    Lt(Key, Value),
    Lte(Key, Value),
    Gt(Key, Value),
    Gte(Key, Value),
    And(Vec<Query>),
    Or(Vec<Query>),
    All,
}

impl Query {
    #[allow(dead_code)]
    pub fn eq(key: &str, value: Value) -> Self {
        Self::Eq(Key(key.to_string()), value)
    }

    #[allow(dead_code)]
    pub fn ne(key: &str, value: Value) -> Self {
        Self::Ne(Key(key.to_string()), value)
    }

    #[allow(dead_code)]
    pub fn and(queries: Vec<Query>) -> Self {
        Self::And(queries)
    }

    #[allow(dead_code)]
    pub fn or(queries: Vec<Query>) -> Self {
        Self::Or(queries)
    }

    #[allow(dead_code)]
    pub fn like(key: &str, value: String) -> Self {
        Self::Like(Key(key.to_string()), value)
    }

    #[allow(dead_code)]
    pub fn lt(key: &str, value: Value) -> Self {
        Self::Lt(Key(key.to_string()), value)
    }

    #[allow(dead_code)]
    pub fn lte(key: &str, value: Value) -> Self {
        Self::Lte(Key(key.to_string()), value)
    }

    #[allow(dead_code)]
    pub fn gt(key: &str, value: Value) -> Self {
        Self::Gt(Key(key.to_string()), value)
    }

    #[allow(dead_code)]
    pub fn gte(key: &str, value: Value) -> Self {
        Self::Gte(Key(key.to_string()), value)
    }

    #[allow(dead_code)]
    pub fn all() -> Self {
        Self::All
    }

    fn get_value(&self, value: &Value, key: &str) -> Option<Value> {
        if !key.contains('.') {
            return value.get(key).cloned();
        }
        let mut keys = key.split('.').peekable();
        let mut value = value;
        while let Some(key) = keys.next() {
            if let Some(nested) = value.get(key) {
                value = nested;
            } else {
                return None;
            }
        }
        Some(value.clone())
    }

    pub fn matches(&self, value: &Value) -> Result<bool, anyhow::Error> {
        let is_match = match self {
            Self::Eq(key, query_value) => {
                let value = self.get_value(value, &key.0);
                value == Some(query_value.clone())
            }
            Self::Ne(key, query_value) => {
                let value = self.get_value(value, &key.0);
                value != Some(query_value.clone())
            }
            Self::Like(key, query_value) => {
                let value = self.get_value(value, &key.0);
                if let Some(value) = value {
                    if let Some(value) = value.as_str() {
                        value.contains(query_value)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Lt(key, query_value) => {
                let value = self.get_value(value, &key.0);
                if let Some(value) = value {
                    if let Some(value) = value.as_f64() {
                        let query_value = query_value.as_f64();
                        match query_value {
                            Some(query_value) => value < query_value,
                            None => false,
                        }
                    } else if let Some(value) = value.as_i64() {
                        let query_value = query_value.as_i64();
                        match query_value {
                            Some(query_value) => value < query_value,
                            None => false,
                        }
                    } else if let Some(value) = value.as_u64() {
                        let query_value = query_value.as_u64();
                        match query_value {
                            Some(query_value) => value < query_value,
                            None => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Lte(key, query_value) => {
                let value = self.get_value(value, &key.0);
                if let Some(value) = value {
                    if let Some(value) = value.as_f64() {
                        let query_value = query_value.as_f64();
                        match query_value {
                            Some(query_value) => value <= query_value,
                            None => false,
                        }
                    } else if let Some(value) = value.as_i64() {
                        let query_value = query_value.as_i64();
                        match query_value {
                            Some(query_value) => value <= query_value,
                            None => false,
                        }
                    } else if let Some(value) = value.as_u64() {
                        let query_value = query_value.as_u64();
                        match query_value {
                            Some(query_value) => value <= query_value,
                            None => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Gt(key, query_value) => {
                let value = self.get_value(value, &key.0);
                if let Some(value) = value {
                    if let Some(value) = value.as_f64() {
                        let query_value = query_value.as_f64();
                        match query_value {
                            Some(query_value) => value > query_value,
                            None => false,
                        }
                    } else if let Some(value) = value.as_i64() {
                        let query_value = query_value.as_i64();
                        match query_value {
                            Some(query_value) => value > query_value,
                            None => false,
                        }
                    } else if let Some(value) = value.as_u64() {
                        let query_value = query_value.as_u64();
                        match query_value {
                            Some(query_value) => value > query_value,
                            None => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Gte(key, query_value) => {
                let value = self.get_value(value, &key.0);
                if let Some(value) = value {
                    if let Some(value) = value.as_f64() {
                        let query_value = query_value.as_f64();
                        match query_value {
                            Some(query_value) => value >= query_value,
                            None => false,
                        }
                    } else if let Some(value) = value.as_i64() {
                        let query_value = query_value.as_i64();
                        match query_value {
                            Some(query_value) => value >= query_value,
                            None => false,
                        }
                    } else if let Some(value) = value.as_u64() {
                        let query_value = query_value.as_u64();
                        match query_value {
                            Some(query_value) => value >= query_value,
                            None => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::And(queries) => queries
                .iter()
                .all(|query| query.matches(value).unwrap_or_else(|_| false)),
            Self::Or(queries) => queries
                .iter()
                .any(|query| query.matches(value).unwrap_or_else(|_| false)),
            Self::All => true,
        };
        Ok(is_match)
    }
}
