use std::collections::HashMap;

use super::{index::{value_to_key, BuiltIndex, IndexKey, ValueKey}, query::Query};

#[derive(Debug, Clone)]
pub enum Constraint {
    Eq(ValueKey),
    Range {
        min: Option<ValueKey>,
        max: Option<ValueKey>,
    },
}

impl Constraint {
    pub fn merge(&self, other: &Constraint) -> Constraint {
        match (self, other) {
            (Constraint::Eq(a), Constraint::Eq(b)) if a == b => Constraint::Eq(a.clone()),
            (
                Constraint::Range {
                    min: a_min,
                    max: a_max,
                },
                Constraint::Range {
                    min: b_min,
                    max: b_max,
                },
            ) => Constraint::Range {
                min: match (a_min, b_min) {
                    (Some(a), Some(b)) => Some(a.clone().max(b.clone())),
                    (Some(a), None) => Some(a.clone()),
                    (None, Some(b)) => Some(b.clone()),
                    _ => None,
                },
                max: match (a_max, b_max) {
                    (Some(a), Some(b)) => Some(a.clone().min(b.clone())),
                    (Some(a), None) => Some(a.clone()),
                    (None, Some(b)) => Some(b.clone()),
                    _ => None,
                },
            },
            (Constraint::Eq(val), Constraint::Range { .. })
            | (Constraint::Range { .. }, Constraint::Eq(val)) => Constraint::Range {
                min: Some(val.clone()),
                max: Some(val.clone()),
            },
            _ => self.clone(),
        }
    }
}

pub fn collect_constraints(query: &Query, constraints: &mut HashMap<String, Constraint>) {
    match query {
        Query::And(subs) => {
            for sub in subs {
                collect_constraints(sub, constraints);
            }
        }
        Query::Eq(field, value) => {
            if let Some(key) = value_to_key(value) {
                constraints
                    .entry(field.clone().to_string())
                    .and_modify(|c| *c = c.merge(&Constraint::Eq(key.clone())))
                    .or_insert(Constraint::Eq(key));
            }
        }
        Query::Gt(field, value) => {
            if let Some(key) = value_to_key(value) {
                constraints
                    .entry(field.clone().to_string())
                    .and_modify(|c| {
                        *c = c.merge(&Constraint::Range {
                            min: Some(key.clone()),
                            max: None,
                        })
                    })
                    .or_insert(Constraint::Range {
                        min: Some(key),
                        max: None,
                    });
            }
        }
        Query::Lt(field, value) => {
            if let Some(key) = value_to_key(value) {
                constraints
                    .entry(field.clone().to_string())
                    .and_modify(|c| {
                        *c = c.merge(&Constraint::Range {
                            min: None,
                            max: Some(key.clone()),
                        })
                    })
                    .or_insert(Constraint::Range {
                        min: None,
                        max: Some(key),
                    });
            }
        }
        _ => {}
    }
}

pub fn query_with_index(
    built_index: &BuiltIndex,
    constraints: &HashMap<String, Constraint>,
) -> Option<Vec<String>> {
    let mut prefix_keys = Vec::new();
    let mut range_start: Option<IndexKey> = None;
    let mut range_end: Option<IndexKey> = None;

    for col in &built_index.keys {
        if let Some(c) = constraints.get(col) {
            match c {
                Constraint::Eq(v) => {
                    prefix_keys.push(v.clone());
                }
                Constraint::Range { min, max } => {
                    let mut start_parts = prefix_keys.clone();
                    let mut end_parts = prefix_keys.clone();
                    start_parts.push(min.clone().unwrap_or(ValueKey::Null));
                    end_parts.push(
                        max.clone()
                            .unwrap_or(ValueKey::String("\u{10FFFF}".to_string())),
                    );

                    range_start = Some(if start_parts.len() == 1 {
                        IndexKey::Single(start_parts[0].clone())
                    } else {
                        IndexKey::Compound(start_parts)
                    });
                    range_end = Some(if end_parts.len() == 1 {
                        IndexKey::Single(end_parts[0].clone())
                    } else {
                        IndexKey::Compound(end_parts)
                    });
                    break;
                }
            }
        } else {
            break;
        }
    }

    if let (Some(start), Some(end)) = (range_start, range_end) {
        Some(
            built_index
                .map
                .range(start..=end)
                .flat_map(|(_, ids)| ids.clone())
                .collect(),
        )
    } else if !prefix_keys.is_empty() {
        let key = if prefix_keys.len() == 1 {
            IndexKey::Single(prefix_keys[0].clone())
        } else {
            IndexKey::Compound(prefix_keys)
        };
        Some(built_index.map.get(&key).cloned().unwrap_or_default())
    } else {
        None
    }
}
