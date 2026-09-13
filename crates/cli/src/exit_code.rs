pub mod exit_code {
    pub const OK: i32 = 0;
    pub const GENERIC: i32 = 1;
    pub const USAGE: i32 = 2;
    pub const VALIDATION: i32 = 3;
    pub const POLICY: i32 = 4;
    pub const NOT_FOUND: i32 = 5;
    pub const CONFLICT: i32 = 6;
    pub const CANCELLED: i32 = 7;
    pub const INTERNAL: i32 = 70;

    pub fn from_config_error(err: &config::ConfigError) -> i32 {
        match err {
            config::ConfigError::Validation(_) => VALIDATION,
            config::ConfigError::MissingKey(_) => NOT_FOUND,
            config::ConfigError::InterpolationCycle(_) => VALIDATION,
            config::ConfigError::FileRead { .. } => GENERIC,
            config::ConfigError::Parse(_) => VALIDATION,
        }
    }

    pub fn from_storage_error(err: &storage::StorageError) -> i32 {
        match err {
            storage::StorageError::NotFound { .. } => NOT_FOUND,
            storage::StorageError::Conflict => CONFLICT,
            storage::StorageError::ImmutableSourceVersion => POLICY,
            storage::StorageError::ConstraintViolation { .. } => VALIDATION,
            storage::StorageError::StorageBusy => GENERIC,
            storage::StorageError::MigrationRequired { .. } => VALIDATION,
            storage::StorageError::MigrationChecksumMismatch { .. } => VALIDATION,
            storage::StorageError::IdempotencyKeyReplay => CONFLICT,
            storage::StorageError::StorageUnavailable => INTERNAL,
        }
    }

    pub fn from_io_error(err: &std::io::Error) -> i32 {
        match err.kind() {
            std::io::ErrorKind::NotFound => NOT_FOUND,
            std::io::ErrorKind::PermissionDenied => POLICY,
            std::io::ErrorKind::AlreadyExists => CONFLICT,
            _ => GENERIC,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::exit_code;
    use config::ConfigError;
    use storage::StorageError;

    #[test]
    fn config_validation_maps_to_validation() {
        let err = ConfigError::Validation("bad".into());
        assert_eq!(exit_code::from_config_error(&err), exit_code::VALIDATION);
    }

    #[test]
    fn config_missing_key_maps_to_not_found() {
        let err = ConfigError::MissingKey("x".into());
        assert_eq!(exit_code::from_config_error(&err), exit_code::NOT_FOUND);
    }

    #[test]
    fn storage_not_found_maps_to_not_found() {
        let err = StorageError::NotFound { urn: "x".into() };
        assert_eq!(exit_code::from_storage_error(&err), exit_code::NOT_FOUND);
    }

    #[test]
    fn storage_conflict_maps_to_conflict() {
        let err = StorageError::Conflict;
        assert_eq!(exit_code::from_storage_error(&err), exit_code::CONFLICT);
    }

    #[test]
    fn storage_busy_maps_to_generic() {
        let err = StorageError::StorageBusy;
        assert_eq!(exit_code::from_storage_error(&err), exit_code::GENERIC);
    }

    #[test]
    fn constants_are_correct() {
        assert_eq!(exit_code::OK, 0);
        assert_eq!(exit_code::GENERIC, 1);
        assert_eq!(exit_code::USAGE, 2);
        assert_eq!(exit_code::VALIDATION, 3);
        assert_eq!(exit_code::POLICY, 4);
        assert_eq!(exit_code::NOT_FOUND, 5);
        assert_eq!(exit_code::CONFLICT, 6);
        assert_eq!(exit_code::CANCELLED, 7);
        assert_eq!(exit_code::INTERNAL, 70);
    }
}
