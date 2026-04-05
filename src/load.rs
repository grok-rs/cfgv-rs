use std::path::Path;

use crate::error::{ValidationError, validate_context};
use crate::schema::Schema;
use crate::value::Value;

/// Load a configuration file, validate it, and apply defaults.
///
/// # Arguments
/// - `filename` — path to the file to load
/// - `schema` — the schema to validate against
/// - `load_strategy` — a function that parses file contents into a `Value`
/// - `display_filename` — optional display name for error messages (defaults to `filename`)
///
/// # Errors
/// Returns `ValidationError` if the file doesn't exist, can't be read as UTF-8,
/// fails parsing, or fails validation.
pub fn load_from_filename<F>(
    filename: &Path,
    schema: &Schema,
    load_strategy: F,
    display_filename: Option<&str>,
) -> Result<Value, ValidationError>
where
    F: Fn(&str) -> Result<Value, Box<dyn std::error::Error>>,
{
    let display = display_filename
        .map(|s| s.to_owned())
        .unwrap_or_else(|| filename.display().to_string());

    if !filename.is_file() {
        return Err(ValidationError::new(format!("{display} is not a file")));
    }

    validate_context(
        || format!("File {display}"),
        || {
            let contents = std::fs::read_to_string(filename)
                .map_err(|e| ValidationError::new(e.to_string()))?;

            let data = load_strategy(&contents).map_err(|e| ValidationError::new(e.to_string()))?;

            schema.check(&data)?;
            Ok(schema.apply_defaults(&data))
        },
    )
}
