use std::fmt;

/// A validation error with an optional context chain.
///
/// Errors can be nested to produce human-readable traces like:
/// ```text
/// ==> File config.yaml
/// ==> At Repo(url='https://...')
/// ==> At key: hooks
/// =====> Missing required key: id
/// ```
#[derive(Debug)]
pub struct ValidationError {
    message: String,
    ctx: Option<Box<ValidationError>>,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        ValidationError {
            message: message.into(),
            ctx: None,
        }
    }

    /// Returns the error message at this level of the context chain.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Wrap this error with additional context, producing a new error
    /// where `ctx` points to `self`.
    pub fn with_context(self, ctx: impl Into<String>) -> Self {
        ValidationError {
            message: ctx.into(),
            ctx: Some(Box::new(self)),
        }
    }

    /// Walk the context chain and collect all parts.
    /// Returns (contexts, leaf_message).
    fn trace(&self) -> (Vec<&str>, &str) {
        let mut contexts = Vec::new();
        let mut current = self;
        while let Some(inner) = &current.ctx {
            contexts.push(current.message.as_str());
            current = inner;
        }
        (contexts, &current.message)
    }

    /// Returns the parts of the error trace as a tuple of strings,
    /// useful for testing.
    pub fn trace_parts(&self) -> Vec<&str> {
        let (contexts, leaf) = self.trace();
        let mut parts = contexts;
        parts.push(leaf);
        parts
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        let (contexts, leaf) = self.trace();
        for ctx in &contexts {
            writeln!(f, "==> {ctx}")?;
        }
        write!(f, "=====> {leaf}")
    }
}

impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.ctx.as_deref().map(|e| e as &dyn std::error::Error)
    }
}

/// Run a closure, and if it returns a `ValidationError`, wrap it with context.
///
/// The context is provided as a closure to avoid allocating a `String`
/// on the success path.
pub fn validate_context<T, F, C>(ctx: C, f: F) -> Result<T, ValidationError>
where
    C: FnOnce() -> String,
    F: FnOnce() -> Result<T, ValidationError>,
{
    f().map_err(|e| e.with_context(ctx()))
}
