# cfgv

[![CI](https://github.com/grok-rs/cfgv-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/grok-rs/cfgv-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/cfgv.svg)](https://crates.io/crates/cfgv)
[![docs.rs](https://docs.rs/cfgv/badge.svg)](https://docs.rs/cfgv)
[![License: MIT](https://img.shields.io/crates/l/cfgv.svg)](LICENSE)

Validate configuration and produce human readable error messages.

A Rust port of the Python [cfgv](https://github.com/asottile/cfgv) library.

## Installation

```toml
[dependencies]
cfgv = "0.1"
```

To enable `From<serde_json::Value>` conversion:

```toml
[dependencies]
cfgv = { version = "0.1", features = ["serde_json"] }
```

## Usage

```rust
use cfgv::{Schema, Validator, CheckFn, Value, value_map, validate, apply_defaults};

let schema = Schema::map(
    "Repo", Some(Value::from("url")),
    vec![
        Validator::required("url", CheckFn::check_string()),
        Validator::optional("rev", CheckFn::check_string(), "HEAD"),
    ],
);

let config = value_map! {
    "url" => "https://github.com/example/repo",
};

validate(&config, &schema).unwrap();

let with_defaults = apply_defaults(&config, &schema);
// with_defaults now has "rev" set to "HEAD"
```

## Error Messages

Validation errors produce human-readable nested traces:

```text
==> File .pre-commit-config.yaml
==> At Config()
==> At key: repos
==> At Repository(repo='https://github.com/pre-commit/pre-commit-hooks')
==> At key: hooks
==> At Hook(id='flake8')
==> At key: always_run
=====> Expected bool got str
```

## Schema Building Blocks

### Containers

| Type | Description |
|------|-------------|
| `Schema::map(name, id_key, items)` | Validates a map against a list of validators |
| `Schema::array(schema)` | Validates a list where each element matches a schema |
| `Schema::array_nonempty(schema)` | Same as `array` but requires at least one element |
| `Schema::key_value_map(name, key_check, value_schema)` | Validates a homogeneous mapping |

### Validators

| Type | Description |
|------|-------------|
| `Validator::required(key, check_fn)` | Key must be present and pass check |
| `Validator::required_recurse(key, schema)` | Key must be present, validated recursively |
| `Validator::optional(key, check_fn, default)` | Checked if present; default applied if missing |
| `Validator::optional_recurse(key, schema, default)` | Optional with recursive schema validation |
| `Validator::optional_no_default(key, check_fn)` | Optional check, no default handling |
| `Validator::conditional(key, check_fn, cond_key, cond_val)` | Checked only when condition matches |
| `Validator::conditional_recurse(key, schema, cond_key, cond_val)` | Recursive check when condition matches |
| `Validator::no_additional_keys(keys)` | Errors if unexpected keys are present |

### Check Functions

| Function | Description |
|----------|-------------|
| `CheckFn::Any` | Accepts any value |
| `CheckFn::check_bool()` | Validates boolean |
| `CheckFn::check_int()` | Validates integer |
| `CheckFn::check_string()` | Validates string |
| `CheckFn::check_regex()` | Validates regex pattern |
| `CheckFn::check_one_of(values)` | Value must be in set |
| `CheckFn::check_array(inner)` | Validates list with inner check |
| `CheckFn::check_and(fns)` | Composes multiple checks |
| `CheckFn::custom(fn)` | User-provided check function |

### Condition Values

Used with conditional validators to control when checks fire:

| Type | Description |
|------|-------------|
| `ConditionValue::Exact(val)` | Matches when equal to `val` |
| `ConditionValue::Not(val)` | Matches when not equal to `val` |
| `ConditionValue::In(vals)` | Matches when value is in set |
| `ConditionValue::NotIn(vals)` | Matches when value is not in set |

## Top-Level Functions

- **`validate(value, schema)`** -- Check value against schema, returns `Result`
- **`apply_defaults(value, schema)`** -- Returns new value with missing optionals set to defaults
- **`remove_defaults(value, schema)`** -- Returns new value with default-valued optionals removed
- **`load_from_filename(path, schema, load_strategy, display_name)`** -- Load file, validate, apply defaults

## License

MIT
