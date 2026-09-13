//! CLI exit-code constants and error mapping (plan D0.13 conventions).

/// Successful execution.
pub const OK: i32 = 0;
/// Generic failure.
pub const GENERIC: i32 = 1;
/// Usage error (bad flags/args).
pub const USAGE: i32 = 2;
/// Validation failure.
pub const VALIDATION: i32 = 3;
/// Denied by policy.
pub const POLICY: i32 = 4;
/// Resource not found.
pub const NOT_FOUND: i32 = 5;
/// Conflict / invalid state.
pub const CONFLICT: i32 = 6;
/// Operation cancelled.
pub const CANCELLED: i32 = 7;
/// Internal error.
pub const INTERNAL: i32 = 70;

/// Map a config error to an exit code.
pub fn from_config_error(err: &config::ConfigError) -> i32 {
    match err {
        config::ConfigError::Validation(_) => VALIDATION,
        config::ConfigError::MissingKey(_) => NOT_FOUND,
        config::ConfigError::InterpolationCycle(_) => VALIDATION,
        config::ConfigError::FileRead { .. } => GENERIC,
        config::ConfigError::Parse(_) => VALIDATION,
    }
}

/// Map an I/O error to an exit code.
pub fn from_io_error(err: &std::io::Error) -> i32 {
    match err.kind() {
        std::io::ErrorKind::NotFound => NOT_FOUND,
        std::io::ErrorKind::PermissionDenied => POLICY,
        std::io::ErrorKind::AlreadyExists => CONFLICT,
        _ => GENERIC,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::ConfigError;

    #[test]
    fn constants_are_correct() {
        assert_eq!(OK, 0);
        assert_eq!(GENERIC, 1);
        assert_eq!(USAGE, 2);
        assert_eq!(VALIDATION, 3);
        assert_eq!(POLICY, 4);
        assert_eq!(NOT_FOUND, 5);
        assert_eq!(CONFLICT, 6);
        assert_eq!(CANCELLED, 7);
        assert_eq!(INTERNAL, 70);
    }

    #[test]
    fn config_validation_maps_to_validation() {
        let err = ConfigError::Validation("bad".into());
        assert_eq!(from_config_error(&err), VALIDATION);
    }

    #[test]
    fn config_missing_key_maps_to_not_found() {
        let err = ConfigError::MissingKey("x".into());
        assert_eq!(from_config_error(&err), NOT_FOUND);
    }
}
