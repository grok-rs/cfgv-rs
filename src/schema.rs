use indexmap::IndexMap;

use crate::check::CheckFn;
use crate::error::{ValidationError, validate_context};
use crate::validator::Validator;
use crate::value::Value;

/// A schema that validates configuration data structures.
///
/// Mirrors Python cfgv's `Map`, `Array`, and `KeyValueMap` containers.
#[derive(Debug, Clone)]
pub enum Schema {
    /// Validates a map (dict) against a set of validators.
    Map {
        object_name: String,
        id_key: Option<Value>,
        items: Vec<Validator>,
    },
    /// Validates a list where each element matches a sub-schema.
    Array { of: Box<Schema>, allow_empty: bool },
    /// Validates a homogeneous mapping with typed keys and values matching a schema.
    KeyValueMap {
        object_name: String,
        check_key_fn: CheckFn,
        value_schema: Box<Schema>,
    },
}

impl Schema {
    /// Validate a value against this schema.
    pub fn check(&self, value: &Value) -> Result<(), ValidationError> {
        match self {
            Schema::Map {
                object_name,
                id_key,
                items,
            } => {
                let dct = match value {
                    Value::Map(m) => m,
                    _ => {
                        return Err(ValidationError::new(format!(
                            "Expected a {object_name} map but got a {}",
                            value.type_name()
                        )));
                    }
                };

                let context = || {
                    if let Some(id_key) = id_key {
                        let key_v = dct
                            .get(id_key)
                            .map(|v| v.repr())
                            .unwrap_or_else(|| "MISSING".to_owned());
                        format!("At {object_name}({id_key}={key_v})")
                    } else {
                        format!("At {object_name}()")
                    }
                };

                validate_context(context, || {
                    for item in items {
                        item.check(dct)?;
                    }
                    Ok(())
                })
            }

            Schema::Array { of, allow_empty } => {
                // First check it's actually an array
                let list = match value {
                    Value::List(l) => l,
                    _ => {
                        return Err(ValidationError::new(format!(
                            "Expected array but got '{}'",
                            value.type_name()
                        )));
                    }
                };

                if !allow_empty && list.is_empty() {
                    let name = match of.as_ref() {
                        Schema::Map { object_name, .. } => object_name.as_str(),
                        Schema::KeyValueMap { object_name, .. } => object_name.as_str(),
                        _ => "item",
                    };
                    return Err(ValidationError::new(format!(
                        "Expected at least 1 '{name}'"
                    )));
                }

                for val in list {
                    of.check(val)?;
                }
                Ok(())
            }

            Schema::KeyValueMap {
                object_name,
                check_key_fn,
                value_schema,
            } => {
                let dct = match value {
                    Value::Map(m) => m,
                    _ => {
                        return Err(ValidationError::new(format!(
                            "Expected a {object_name} map but got a {}",
                            value.type_name()
                        )));
                    }
                };

                validate_context(
                    || format!("At {object_name}()"),
                    || {
                        for (k, val) in dct {
                            validate_context(|| format!("For key: {k}"), || check_key_fn.check(k))?;
                            validate_context(
                                || format!("At key: {k}"),
                                || value_schema.check(val),
                            )?;
                        }
                        Ok(())
                    },
                )
            }
        }
    }

    /// Apply defaults to a value according to this schema.
    /// Returns a new value — does not mutate the input.
    pub fn apply_defaults(&self, value: &Value) -> Value {
        match self {
            Schema::Map { items, .. } => {
                let mut dct = value.as_map().cloned().unwrap_or_default();
                for item in items {
                    item.apply_default(&mut dct);
                }
                Value::Map(dct)
            }

            Schema::Array { of, .. } => {
                let list = value.as_list().map(|l| l.to_vec()).unwrap_or_default();
                Value::List(list.into_iter().map(|v| of.apply_defaults(&v)).collect())
            }

            Schema::KeyValueMap { value_schema, .. } => {
                let dct = value.as_map().cloned().unwrap_or_default();
                let new_map: IndexMap<Value, Value> = dct
                    .into_iter()
                    .map(|(k, v)| (k, value_schema.apply_defaults(&v)))
                    .collect();
                Value::Map(new_map)
            }
        }
    }

    /// Remove defaults from a value according to this schema.
    /// Returns a new value — does not mutate the input.
    pub fn remove_defaults(&self, value: &Value) -> Value {
        match self {
            Schema::Map { items, .. } => {
                let mut dct = value.as_map().cloned().unwrap_or_default();
                for item in items {
                    item.remove_default(&mut dct);
                }
                Value::Map(dct)
            }

            Schema::Array { of, .. } => {
                let list = value.as_list().map(|l| l.to_vec()).unwrap_or_default();
                Value::List(list.into_iter().map(|v| of.remove_defaults(&v)).collect())
            }

            Schema::KeyValueMap { value_schema, .. } => {
                let dct = value.as_map().cloned().unwrap_or_default();
                let new_map: IndexMap<Value, Value> = dct
                    .into_iter()
                    .map(|(k, v)| (k, value_schema.remove_defaults(&v)))
                    .collect();
                Value::Map(new_map)
            }
        }
    }

    // --- Convenience constructors ---

    pub fn map(
        object_name: impl Into<String>,
        id_key: Option<Value>,
        items: Vec<Validator>,
    ) -> Self {
        Schema::Map {
            object_name: object_name.into(),
            id_key,
            items,
        }
    }

    pub fn array(of: Schema) -> Self {
        Schema::Array {
            of: Box::new(of),
            allow_empty: true,
        }
    }

    pub fn array_nonempty(of: Schema) -> Self {
        Schema::Array {
            of: Box::new(of),
            allow_empty: false,
        }
    }

    pub fn key_value_map(
        object_name: impl Into<String>,
        check_key_fn: CheckFn,
        value_schema: Schema,
    ) -> Self {
        Schema::KeyValueMap {
            object_name: object_name.into(),
            check_key_fn,
            value_schema: Box::new(value_schema),
        }
    }
}
