use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entity::Entity;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Key(String);

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    Associated(Entity, Box<Query>),
    All,
}

impl Query {
    /// Create a new query that matches documents based on exact match.
    ///
    /// ```
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
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
    /// use deeb_core::database::query::Query;
    /// let query = Query::all();
    /// ```
    #[allow(dead_code)]
    pub fn all() -> Self {
        Self::All
    }

    /// Create a new query that matches documents based on associated entity.
    /// ```
    /// use deeb_core::database::query::Query;
    /// use deeb_core::entity::Entity;
    /// let user = Entity::new("user");
    /// let comment = Entity::new("comment");
    /// let query = Query::associated(comment, Query::eq("user_id", 1));
    /// ```
    #[allow(dead_code)]
    pub fn associated(entity: Entity, query: Query) -> Self {
        Self::Associated(entity, Box::new(query))
    }

    fn get_kv(&self, value: &Value, key: &str) -> Option<(Key, Value)> {
        if !key.contains('.') {
            let value = value.get(key);
            if value.is_none() {
                return None;
            }
            return Some((Key(key.to_string()), value.cloned().unwrap()));
        }
        let mut keys = key.split('.').peekable();
        let mut value = value;
        let mut current_key = None;
        while let Some(key) = keys.next() {
            current_key = Some(key.to_string());
            if !value.is_object() {
                break;
            }
            if let Some(nested) = value.get(key) {
                value = nested;
            } else {
                return None;
            }
        }
        Some((Key(current_key.unwrap()), value.clone()))
    }

    pub fn associated_entities(&self) -> Vec<Entity> {
        let mut entities = vec![];
        match self {
            Self::Associated(entity, query) => {
                entities.push(entity.clone());
                entities.append(&mut query.associated_entities());
            }
            Self::And(queries) => {
                for query in queries {
                    entities.append(&mut query.associated_entities());
                }
            }
            Self::Or(queries) => {
                for query in queries {
                    entities.append(&mut query.associated_entities());
                }
            }
            _ => {}
        }
        entities
    }

    /// Check if the query matches the value.
    ///
    /// ```
    /// use deeb_core::database::query::Query;
    /// use serde_json::json;
    /// let query = Query::eq("name", "John");
    /// let value = json!({"name": "John"});
    /// let is_match = query.matches(&value).unwrap();
    /// assert_eq!(is_match, true);
    /// ```
    pub fn matches(&self, value: &Value) -> Result<bool, anyhow::Error> {
        let is_match = match self {
            Self::Eq(key, query_value) => {
                let kv = self.get_kv(value, &key.0);
                if let Some((kv_key, value)) = kv {
                    if value.is_array() {
                        let value = value.as_array().unwrap();
                        for v in value {
                            if v.is_object() {
                                let v = v.as_object().unwrap();
                                for (k, v) in v.iter() {
                                    if v == query_value && k == &kv_key.to_string() {
                                        return Ok(true);
                                    }
                                }
                            }
                            if v == query_value {
                                return Ok(true);
                            }
                        }
                        return Ok(false);
                    }
                    value == query_value.clone()
                } else {
                    false
                }
            }
            Self::Ne(key, query_value) => {
                let kv = self.get_kv(value, &key.0);
                if let Some((_key, value)) = kv {
                    if value.is_array() {
                        let value = value.as_array().unwrap();
                        for v in value {
                            if v.is_object() {
                                let v = v.as_object().unwrap();
                                for (k, v) in v.iter() {
                                    if v == query_value && k == &key.0 {
                                        return Ok(false);
                                    }
                                }
                                return Ok(true);
                            }
                            if v == query_value {
                                return Ok(false);
                            }
                        }
                    }
                    value != query_value.clone()
                } else {
                    false
                }
            }
            Self::Like(key, query_value) => {
                let kv = self.get_kv(value, &key.0);
                if let Some((key, value)) = kv {
                    if value.is_array() {
                        let value = value.as_array().unwrap();
                        for v in value {
                            if v.is_object() {
                                let v = v.as_object().unwrap();
                                for (k, v) in v.iter() {
                                    if let Some(value) = v.as_str() {
                                        if value.contains(query_value) && k == &key.to_string() {
                                            return Ok(true);
                                        }
                                    }
                                }
                            }
                            if let Some(value) = v.as_str() {
                                if value.contains(query_value) {
                                    return Ok(true);
                                }
                            }
                        }
                        return Ok(false);
                    }
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
                let kv = self.get_kv(value, &key.0);
                if let Some((key, value)) = kv {
                    // Handle Array
                    if value.is_array() {
                        let value = value.as_array().unwrap();
                        for v in value {
                            if v.is_object() {
                                let v = v.as_object().unwrap();
                                for (k, v) in v.iter() {
                                    if let Some(value) = v.as_f64() {
                                        let query_value = query_value.as_f64();
                                        if query_value.is_none() {
                                            continue;
                                        }
                                        let is_lt = value < query_value.unwrap() && k == &key.0;
                                        if is_lt {
                                            return Ok(true);
                                        }
                                    }
                                }
                            }
                            if let Some(value) = v.as_f64() {
                                let query_value = query_value.as_f64();
                                if query_value.is_none() {
                                    continue;
                                }
                                let is_lt = value < query_value.unwrap();
                                if is_lt {
                                    return Ok(true);
                                }
                            }
                        }
                        return Ok(false);
                    }
                    // Handle primitive types
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
                let kv = self.get_kv(value, &key.0);
                if let Some((key, value)) = kv {
                    // Handle Array
                    if value.is_array() {
                        let value = value.as_array().unwrap();
                        for v in value {
                            if v.is_object() {
                                let v = v.as_object().unwrap();
                                for (k, v) in v.iter() {
                                    if let Some(value) = v.as_f64() {
                                        let query_value = query_value.as_f64();
                                        if query_value.is_none() {
                                            continue;
                                        }
                                        let is_lte = value <= query_value.unwrap() && k == &key.0;
                                        if is_lte {
                                            return Ok(true);
                                        }
                                    }
                                }
                            }
                            if let Some(value) = v.as_f64() {
                                let query_value = query_value.as_f64();
                                if query_value.is_none() {
                                    continue;
                                }
                                let is_lte = value <= query_value.unwrap();
                                if is_lte {
                                    return Ok(true);
                                }
                            }
                        }
                        return Ok(false);
                    }

                    // Handle Primitivves
                    if let Some(value) = value.as_f64() {
                        let query_value = query_value.as_f64();
                        match query_value {
                            Some(query_value) => return Ok(value <= query_value),
                            None => return Ok(false),
                        }
                    } else if let Some(value) = value.as_i64() {
                        let query_value = query_value.as_i64();
                        match query_value {
                            Some(query_value) => return Ok(value <= query_value),
                            None => return Ok(false),
                        }
                    } else if let Some(value) = value.as_u64() {
                        let query_value = query_value.as_u64();
                        match query_value {
                            Some(query_value) => return Ok(value <= query_value),
                            None => return Ok(false),
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Self::Gt(key, query_value) => {
                let kv = self.get_kv(value, &key.0);
                if let Some((key, value)) = kv {
                    // handle array
                    if value.is_array() {
                        let value = value.as_array().unwrap();
                        for v in value {
                            if v.is_object() {
                                let v = v.as_object().unwrap();
                                for (k, v) in v.iter() {
                                    if let Some(value) = v.as_f64() {
                                        let query_value = query_value.as_f64();
                                        match query_value {
                                            Some(query_value) => {
                                                return Ok(value > query_value && k == &key.0);
                                            }
                                            None => return Ok(false),
                                        };
                                    }
                                }
                                return Ok(false);
                            }
                            if let Some(value) = v.as_f64() {
                                let query_value = query_value.as_f64();
                                let is_gt = value > query_value.unwrap();
                                if is_gt {
                                    return Ok(true);
                                }
                            }
                        }
                        return Ok(false);
                    }

                    // handle primitives
                    if let Some(value) = value.as_f64() {
                        let query_value = query_value.as_f64();
                        match query_value {
                            Some(query_value) => return Ok(value > query_value),
                            None => return Ok(false),
                        }
                    } else if let Some(value) = value.as_i64() {
                        let query_value = query_value.as_i64();
                        match query_value {
                            Some(query_value) => return Ok(value > query_value),
                            None => return Ok(false),
                        }
                    } else if let Some(value) = value.as_u64() {
                        let query_value = query_value.as_u64();
                        match query_value {
                            Some(query_value) => return Ok(value > query_value),
                            None => return Ok(false),
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Self::Gte(key, query_value) => {
                let kv = self.get_kv(value, &key.0);
                if let Some((key, value)) = kv {
                    // handle array
                    if value.is_array() {
                        let value = value.as_array().unwrap();
                        for v in value {
                            if v.is_object() {
                                let v = v.as_object().unwrap();
                                for (k, v) in v.iter() {
                                    if let Some(value) = v.as_f64() {
                                        let query_value = query_value.as_f64();
                                        match query_value {
                                            Some(query_value) => {
                                                return Ok(value >= query_value && k == &key.0);
                                            }
                                            None => return Ok(false),
                                        };
                                    }
                                }
                                return Ok(false);
                            }
                            if let Some(value) = v.as_f64() {
                                let query_value = query_value.as_f64();
                                if query_value.is_none() {
                                    continue;
                                }
                                let is_gte = value >= query_value.unwrap();
                                if is_gte {
                                    return Ok(true);
                                }
                            }
                        }
                        return Ok(false);
                    }

                    // handle primitives
                    if let Some(value) = value.as_f64() {
                        let query_value = query_value.as_f64();
                        match query_value {
                            Some(query_value) => return Ok(value >= query_value),
                            None => return Ok(false),
                        }
                    } else if let Some(value) = value.as_i64() {
                        let query_value = query_value.as_i64();
                        match query_value {
                            Some(query_value) => return Ok(value >= query_value),
                            None => return Ok(false),
                        }
                    } else if let Some(value) = value.as_u64() {
                        let query_value = query_value.as_u64();
                        match query_value {
                            Some(query_value) => return Ok(value >= query_value),
                            None => return Ok(false),
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Self::And(queries) => queries
                .iter()
                .all(|query| query.matches(value).unwrap_or_else(|_| false)),
            Self::Or(queries) => queries
                .iter()
                .any(|query| query.matches(value).unwrap_or_else(|_| false)),
            Self::Associated(_entity, query) => {
                let is_match = query.matches(value).unwrap_or_else(|_| false);
                is_match
            }
            Self::All => true,
        };
        Ok(is_match)
    }
}
