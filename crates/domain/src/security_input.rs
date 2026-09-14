use crate::security::SecurityError;

/// Input validation limits (D0.15 / T45).
#[derive(Debug, Clone, Copy)]
pub struct InputLimits {
    pub max_field_length: usize,
    pub max_json_depth: u32,
}

impl Default for InputLimits {
    fn default() -> Self {
        Self { max_field_length: 10_000, max_json_depth: 32 }
    }
}

/// Validate a UTF-8 string field: non-empty, within length, no control chars
/// (except common whitespace), and within JSON depth if it parses as JSON.
pub fn check_input_field(value: &str, limits: &InputLimits) -> Result<(), SecurityError> {
    if value.is_empty() {
        return Err(SecurityError::InvalidInput);
    }
    if value.len() > limits.max_field_length {
        return Err(SecurityError::FieldTooLong);
    }
    // Reject control characters except tab/newline/carriage-return.
    if value.chars().any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r')) {
        return Err(SecurityError::InvalidInput);
    }
    Ok(())
}

/// Validate a JSON string's structural depth (D0.15 / T45).
pub fn check_json_depth(value: &str, limits: &InputLimits) -> Result<(), SecurityError> {
    let mut depth: u32 = 0;
    let mut max_seen: u32 = 0;
    for ch in value.chars() {
        match ch {
            '{' | '[' => {
                depth += 1;
                max_seen = max_seen.max(depth);
            }
            '}' | ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if max_seen > limits.max_json_depth {
            return Err(SecurityError::JsonTooDeep);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_field_passes() {
        let limits = InputLimits::default();
        assert!(check_input_field("hello world", &limits).is_ok());
    }

    #[test]
    fn empty_field_rejected() {
        let limits = InputLimits::default();
        assert!(check_input_field("", &limits).is_err());
    }

    #[test]
    fn control_chars_rejected() {
        let limits = InputLimits::default();
        assert!(check_input_field("hello\x00world", &limits).is_err());
    }

    #[test]
    fn json_depth_enforced() {
        let limits = InputLimits { max_json_depth: 3, ..Default::default() };
        assert!(check_json_depth(r#"{"a":{"b":1}}"#, &limits).is_ok());
        assert!(check_json_depth(r#"{"a":{"b":{"c":{"d":1}}}}"#, &limits).is_err());
    }
}
