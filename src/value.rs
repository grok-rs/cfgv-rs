use std::fmt;
use std::hash::{Hash, Hasher};

use indexmap::IndexMap;

/// A dynamically-typed value that can represent configuration data.
///
/// Supports bools, integers, floats, strings, byte arrays, lists, and maps
/// with arbitrary key types (unlike `serde_json::Value` which only allows string keys).
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(IndexMap<Value, Value>),
    Null,
}

impl Value {
    /// Returns the Python-like type name for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "str",
            Value::Bytes(_) => "bytes",
            Value::List(_) => "list",
            Value::Map(_) => "dict",
            Value::Null => "NoneType",
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&IndexMap<Value, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn into_map(self) -> Option<IndexMap<Value, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<Vec<Value>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Bool(b) => b.hash(state),
            Value::Int(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::Bytes(b) => b.hash(state),
            Value::List(l) => l.hash(state),
            Value::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::Null => {}
        }
    }
}

impl Value {
    /// Python `repr()`-style formatting: strings are quoted with `'...'`.
    pub fn repr(&self) -> String {
        match self {
            Value::Bool(true) => "True".to_owned(),
            Value::Bool(false) => "False".to_owned(),
            Value::Int(i) => format!("{i}"),
            Value::Float(v) => format!("{v}"),
            Value::String(s) => format!("'{s}'"),
            Value::Bytes(b) => format!("{b:?}"),
            Value::List(l) => {
                let inner: Vec<String> = l.iter().map(|v| v.repr()).collect();
                format!("[{}]", inner.join(", "))
            }
            Value::Map(m) => {
                let inner: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.repr(), v.repr()))
                    .collect();
                format!("{{{}}}", inner.join(", "))
            }
            Value::Null => "None".to_owned(),
        }
    }
}

/// Display produces Python `str()`-style output: strings are unquoted.
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(true) => write!(f, "True"),
            Value::Bool(false) => write!(f, "False"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bytes(b) => write!(f, "{b:?}"),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v.repr())?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k.repr(), v.repr())?;
                }
                write!(f, "}}")
            }
            Value::Null => write!(f, "None"),
        }
    }
}

// --- From impls for ergonomic construction ---

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i64::from(i))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_owned())
    }
}

impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Value::Bytes(b)
    }
}

impl From<Vec<Value>> for Value {
    fn from(l: Vec<Value>) -> Self {
        Value::List(l)
    }
}

impl From<IndexMap<Value, Value>> for Value {
    fn from(m: IndexMap<Value, Value>) -> Self {
        Value::Map(m)
    }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::List(arr.into_iter().map(Value::from).collect())
            }
            serde_json::Value::Object(obj) => {
                let map: IndexMap<Value, Value> = obj
                    .into_iter()
                    .map(|(k, v)| (Value::String(k), Value::from(v)))
                    .collect();
                Value::Map(map)
            }
        }
    }
}

#[cfg(feature = "yaml_serde")]
impl From<yaml_serde::Value> for Value {
    fn from(v: yaml_serde::Value) -> Self {
        match v {
            yaml_serde::Value::Null => Value::Null,
            yaml_serde::Value::Bool(b) => Value::Bool(b),
            yaml_serde::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::String(n.to_string())
                }
            }
            yaml_serde::Value::String(s) => Value::String(s),
            yaml_serde::Value::Sequence(arr) => {
                Value::List(arr.into_iter().map(Value::from).collect())
            }
            yaml_serde::Value::Mapping(map) => {
                let m: IndexMap<Value, Value> = map
                    .into_iter()
                    .map(|(k, v)| (Value::from(k), Value::from(v)))
                    .collect();
                Value::Map(m)
            }
            yaml_serde::Value::Tagged(tagged) => Value::from(tagged.value),
        }
    }
}

/// Helper macro to build a `Value::Map` from key-value pairs.
///
/// # Example
///
/// ```
/// use cfgv::{Value, value_map};
///
/// let val = value_map! {
///     "name" => "example",
///     "enabled" => true,
/// };
/// ```
#[macro_export]
macro_rules! value_map {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut map = $crate::__IndexMap::new();
        $(
            map.insert($crate::Value::from($key), $crate::Value::from($val));
        )*
        $crate::Value::Map(map)
    }};
}
