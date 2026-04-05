use crate::value::Value;

/// A condition value used in `Conditional` validators.
///
/// Determines whether a conditional check should fire based on the value
/// of a condition key in the map being validated.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionValue {
    /// Matches when the value equals exactly this value.
    Exact(Value),
    /// Matches when the value is present and not equal to the given value.
    Not(Value),
    /// Matches when the value is present and contained in the set.
    In(Vec<Value>),
    /// Matches when the value is present and not contained in the set.
    NotIn(Vec<Value>),
}

impl ConditionValue {
    /// Check whether `value` matches this condition.
    /// `None` represents a missing key (MISSING in Python).
    pub fn matches(&self, value: Option<&Value>) -> bool {
        match self {
            ConditionValue::Exact(expected) => value.is_some_and(|v| v == expected),
            ConditionValue::Not(excluded) => value.is_some_and(|v| v != excluded),
            ConditionValue::In(values) => value.is_some_and(|v| values.contains(v)),
            ConditionValue::NotIn(values) => value.is_some_and(|v| !values.contains(v)),
        }
    }

    /// Describe the opposite condition, used in `ensure_absent` error messages.
    ///
    /// For `Not(True)`, produces `"is True"`.
    /// For `In(1, 2)`, produces `"is not any of (1, 2)"`.
    pub fn describe_opposite(&self) -> String {
        match self {
            ConditionValue::Exact(val) => format!("is not {val}"),
            ConditionValue::Not(val) => format!("is {val}"),
            ConditionValue::In(values) => {
                let vals = format_value_tuple(values);
                format!("is not any of {vals}")
            }
            ConditionValue::NotIn(values) => {
                let vals = format_value_tuple(values);
                format!("is any of {vals}")
            }
        }
    }
}

fn format_value_tuple(values: &[Value]) -> String {
    let inner: Vec<String> = values.iter().map(|v| format!("{v}")).collect();
    format!("({})", inner.join(", "))
}
