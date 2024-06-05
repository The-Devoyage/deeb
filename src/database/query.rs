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
    /// Create a new query that matches documents based on exact match.
    ///
    /// ```
    /// use deeb::*;
    ///
    /// let query = Query::eq("name", "John");
    /// ```
    #[allow(dead_code)]
    pub fn eq<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        Self::Eq(key.into(), value.into())
    }

    /// Create a new query that matches documents based on not equal match.
    ///
    /// ```
    /// use deeb::*;
    /// let query = Query::ne("name", "John");
    /// ```
    #[allow(dead_code)]
    pub fn ne<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        Self::Ne(key.into(), value.into())
    }

    /// Create a new query that matches documents based on multiple conditions.
    ///
    /// ```
    /// use deeb::*;
    ///
    /// let query = Query::and(vec![
    ///    Query::eq("name", "John"),
    ///    Query::eq("age", 30),
    ///    Query::eq("city", "New York"),
    /// ]);
    /// ```
    #[allow(dead_code)]
    pub fn and(queries: Vec<Query>) -> Self {
        Self::And(queries)
    }

    /// Create a new query that matches documents based on multiple conditions.
    ///
    /// ```
    /// use deeb::*;
    ///
    /// let query = Query::or(vec![
    ///   Query::eq("name", "John"),
    ///   Query::eq("age", 30),
    ///  Query::eq("city", "New York"),
    /// ]);
    /// ```
    #[allow(dead_code)]
    pub fn or(queries: Vec<Query>) -> Self {
        Self::Or(queries)
    }

    /// Create a new query that matches documents based on like match.
    ///
    /// ```
    /// use deeb::*;
    /// let query = Query::like("name", "John");
    /// ```
    #[allow(dead_code)]
    pub fn like<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<String>,
    {
        Self::Like(key.into(), value.into())
    }

    /// Create a new query that matches documents based on less than match.
    ///
    /// ```
    /// use deeb::*;
    /// let query = Query::lt("age", 30);
    /// ```
    #[allow(dead_code)]
    pub fn lt<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        Self::Lt(key.into(), value.into())
    }

    /// Create a new query that matches documents based on less than or equal match.
    ///
    /// ```
    /// use deeb::*;
    /// let query = Query::lte("age", 30);
    /// ```
    #[allow(dead_code)]
    pub fn lte<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        Self::Lte(key.into(), value.into())
    }

    /// Create a new query that matches documents based on greater than match.
    ///
    /// ```
    /// use deeb::*;
    /// let query = Query::gt("age", 30);
    /// ```
    #[allow(dead_code)]
    pub fn gt<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        Self::Gt(key.into(), value.into())
    }

    /// Create a new query that matches documents based on greater than or equal match.
    ///
    /// ```
    /// use deeb::*;
    /// let query = Query::gte("age", 30);
    /// ```
    #[allow(dead_code)]
    pub fn gte<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        Self::Gte(key.into(), value.into())
    }

    /// Create a new query that matches all documents.
    /// ```
    /// use deeb::*;
    /// let query = Query::all();
    /// ```
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

    /// Check if the query matches the value.
    ///
    /// ```
    /// use deeb::*;
    /// use serde_json::json;
    /// let query = Query::eq("name", "John");
    /// let value = json!({"name": "John"});
    /// let is_match = query.matches(&value).unwrap();
    /// assert_eq!(is_match, true);
    /// ```
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
